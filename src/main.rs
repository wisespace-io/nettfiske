#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate strsim;
extern crate console;
extern crate tungstenite;
extern crate publicsuffix;
extern crate fern;
extern crate chrono;
extern crate idna;
extern crate unicode_skeleton;

mod util;

use url::Url;
use util::CertString;
use publicsuffix::List;
use tungstenite::{connect};
use serde_json::{from_str};
use console::{Emoji, style};
use strsim::{damerau_levenshtein};
use idna::punycode::{decode};
use unicode_skeleton::{UnicodeSkeleton};

static LOOKING_GLASS: Emoji = Emoji("🔍  ", "");
static WEBSOCKET_URL: &'static str = "wss://certstream.calidog.io";

fn main() {
    match setup_logger() {
        Err(why) => panic!("{}", why),
        Ok(_) => (),
    };

    // Fetch the list from the official URL,
    let mut list = List::fetch().unwrap();

    let wss: String = format!("{}", WEBSOCKET_URL);
    let url = Url::parse(&wss).unwrap();
    let keywords: Vec<_> = util::KEYWORDS.keys().into_iter().collect();

    match connect(url) {
        Ok(mut answer) => {
            println!("{} {} Fetching Certificates ...", style("[Nettfiske]").bold().dim(), LOOKING_GLASS);

            loop {
                let msg = answer.0.read_message().unwrap();      
                if msg.is_text() && msg.len() > 0 {
                    let msg_txt = msg.into_text().unwrap_or("".to_string());
                    if msg_txt.len() > 0 {
                        let cert: CertString = from_str(msg_txt.as_str()).unwrap();

                        if cert.message_type.contains("heartbeat") {
                            continue;
                        } else if cert.message_type.contains("certificate_update") {
                            for mut domain in cert.data.leaf_cert.all_domains {
                                if domain.starts_with("*.") {
                                    domain = domain.replace("*.", "");
                                }
                                             
                                analyse_domain(&domain, &mut list, keywords.clone());
                            }
                        }
                    }
                 }
            }
        }
        Err(e) => {
            println!("Error during handshake {}", style(e).white());
        }
    }
}

fn analyse_domain(original_domain: &str, list: &mut List, keywords: Vec<&&str>) {
    let mut punycode_detected = false;
    let mut score = 0;
    let domain = punycode(original_domain.to_string());
    
    // It means that found punycode
    if original_domain != domain {
        punycode_detected = true;
    }

    if let Ok(domain_obj) = list.parse_domain(&domain) {  

        if let Some(registrable) = domain_obj.root() {
            // Registrable domain
            let domain_name: Vec<&str> = registrable.split('.').collect();

            // Subdomain
            let mut sub_domain = domain.replace(registrable, "");
            sub_domain.pop(); // remove .

            let sub_domain_name: Vec<&str> = sub_domain.split('.').collect();

            for key in &keywords {
                // Check Registration domain
                score += domain_keywords(domain_name[0], key) * 4;
                score += calc_string_edit_distance(domain_name[0], key, 6, punycode_detected);

                // Check subdomain
                for name in &sub_domain_name {
                    score += domain_keywords(name, key) * 5;
                    if !name.contains("mail") && !name.contains("cloud") {
                        score += calc_string_edit_distance(name, key, 4, punycode_detected);
                    }
                }
            }

            // Check for tldl on subdomain
            score += search_tldl_on_subdomain(&sub_domain_name);
        }
    }

    score += deeply_nested(&domain);

    report(score, &original_domain, &domain, punycode_detected);
}

fn search_tldl_on_subdomain(sub_domain: &Vec<&str>) -> usize {
    let tldl: Vec<&str> = vec!["com", "net", "-net", "-com", "net-", "com-", "com/", "net/"];
    for key in &tldl {
        for name in sub_domain {
            if *key == "com" || *key == "net" {
                return domain_keywords_exact_match(&name, key) * 4;
            } else {
                return domain_keywords(&name, key) * 4;
            }
        }
    }
    return 0;
}

fn deeply_nested(domain: &str) -> usize {
    let v: Vec<&str> = domain.split('.').collect();
    let size = if v.len() >= 3 { v.len() * 3 } else { 0 };
    size
}

fn domain_keywords(name: &str, key: &str) -> usize {
    if name.contains(key) {
        return 10;
    }
    return 0;
}

fn domain_keywords_exact_match(name: &str, key: &str) -> usize {
    if name.eq_ignore_ascii_case(key) {
        return 10;
    }
    return 0;
}

// Damerau Levenshtein: Calculates number of operations (Insertions, deletions or substitutions,
// or transposition of two adjacent characters) required to change one word into the other.
fn calc_string_edit_distance(name: &str, key: &str, weight: usize, punycode_detected: bool) -> usize {
    let distance = damerau_levenshtein(name, key);

    if (distance == 1 || distance == 0) && punycode_detected {
        return 15 * weight;
    } else if distance == 1 {
        return 8 * weight;
    }
    return 0;
}

// Decode the domain as Punycode
fn punycode(domain: String) -> String {
    let mut result = Vec::new();
    let words_list: Vec<&str> = domain.split('.').collect();

    for word in &words_list {
        if word.starts_with("xn--") {
            let pu = word.replace("xn--", "");
            let decoded = decode(&pu).unwrap().into_iter().collect::<String>();
            let skeleton = decoded.skeleton_chars().collect::<String>();
            result.push(skeleton.clone());
        } else {
            result.push(word.to_string());
        }
    }

    result.join(".")
}

fn report(score: usize, domain_original: &str, domain: &str, punycode_detected: bool) {
    if score >= 90 && punycode_detected {
        println!("Homoglyph detected {} (Punycode: {})", style(domain).red().on_black().bold(), domain_original);
    } else if score >= 90 {
        println!("Suspicious {} (score {})", style(domain).red(), score);    
    } else if score >= 70 {
        println!("Suspicious {} (score {})", style(domain).yellow(), score);        
    } else if score >= 56 {
        println!("Suspicious {} (score {})", style(domain_original).magenta(), score);
    }

    if score >= 56 {
        if punycode_detected {
            info!("{} - (Punycode: {})", domain, domain_original);
        } else {
            info!("{}", domain);
        }
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, _record| {
            out.finish(format_args!(
                "{} {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file("nettfiske.log")?)
        .apply()?;
    Ok(())
}