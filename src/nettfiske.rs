use crate::data::{Config};
use log::{LevelFilter};
use publicsuffix::List;
use console::{style};
use strsim::{damerau_levenshtein};
use idna::punycode::{decode};
use unicode_skeleton::{UnicodeSkeleton};

pub struct Nettfiske {
    list: List,
    config: Config,
}

impl Nettfiske {
    pub fn new(config: Config) -> Self {
        Nettfiske {
            list: List::fetch().unwrap(),
            config,
        }
    }

    pub fn setup_logger(&self, enable: bool) -> Result<(), fern::InitError> {
        if !enable {
            return Ok(());
        }

        fern::Dispatch::new()
            .format(|out, message, _record| {
                out.finish(format_args!(
                    "{} {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    message
                ))
            })
            .level(LevelFilter::Info)
            .chain(fern::log_file("nettfiske.log")?)
            .apply()?;
        Ok(())
    }

    pub fn analyse_domain(&self, original_domain: &str) {
        let mut punycode_detected = false;
        let mut score = 0;

        let original_domain_str = original_domain.replace("*.", "");

        let domain = self.punycode(original_domain_str.to_string());

        // It means that found punycode
        if original_domain_str != domain {
            punycode_detected = true;
        }

        if let Ok(domain_obj) = self.list.parse_domain(&domain) {
            if let Some(registrable) = domain_obj.root() {
                // Registrable domain
                let domain_name: Vec<&str> = registrable.split('.').collect();

                // Subdomain
                let mut sub_domain = domain.replace(registrable, "");
                sub_domain.pop(); // remove .

                let sub_domain_name: Vec<&str> = sub_domain.split('.').collect();

                for identities in &self.config.identities {
                    let key = identities.common_name.as_str();

                    // Check Registration domain
                    score += self.domain_keywords(domain_name[0], key) * 4;
                    score +=
                        self.calc_string_edit_distance(domain_name[0], key, 6, punycode_detected);

                    // Check subdomain
                    for name in &sub_domain_name {
                        score += self.domain_keywords(name, key) * 5;
                        if !name.contains("mail") && !name.contains("cloud") {
                            score +=
                                self.calc_string_edit_distance(name, key, 4, punycode_detected);
                        }
                    }
                }

                // Check for tldl on subdomain
                score += self.search_tldl_on_subdomain(&sub_domain_name);
            }
        }

        score += self.deeply_nested(&domain);

        self.report(score, &original_domain, &domain, punycode_detected);
    }

    #[allow(clippy::never_loop)]
    fn search_tldl_on_subdomain(&self, sub_domain: &[&str]) -> usize {
        let tldl: Vec<&str> = vec!["com", "net", "-net", "-com", "net-", "com-", "com/", "net/"];
        for key in &tldl {
            for name in sub_domain {
                if *key == "com" || *key == "net" {
                    return self.domain_keywords_exact_match(&name, key) * 4;
                } else {
                    return self.domain_keywords(&name, key) * 4;
                }
            }
        }
        0
    }

    fn deeply_nested(&self, domain: &str) -> usize {
        let v: Vec<&str> = domain.split('.').collect();
        if v.len() >= 3 {
            v.len() * 3
        } else {
            0
        }
    }

    fn domain_keywords(&self, name: &str, key: &str) -> usize {
        if name.contains(key) {
            return 10;
        }
        0
    }

    fn domain_keywords_exact_match(&self, name: &str, key: &str) -> usize {
        if name.eq_ignore_ascii_case(key) {
            return 10;
        }
        0
    }

    // Damerau Levenshtein: Calculates number of operations (Insertions, deletions or substitutions,
    // or transposition of two adjacent characters) required to change one word into the other.
    fn calc_string_edit_distance(
        &self, name: &str, key: &str, weight: usize, punycode_detected: bool,
    ) -> usize {
        let distance = damerau_levenshtein(name, key);

        if (distance == 1 || distance == 0) && punycode_detected {
            return 15 * weight;
        } else if distance == 1 {
            return 8 * weight;
        }
        0
    }

    // Decode the domain as Punycode
    fn punycode(&self, domain: String) -> String {
        let mut result = Vec::new();
        let words_list: Vec<&str> = domain.split('.').collect();

        for word in &words_list {
            if word.starts_with("xn--") {
                let pu = word.replace("xn--", "");
                let decoded = decode(&pu).unwrap().into_iter().collect::<String>();
                let skeleton = decoded.skeleton_chars().collect::<String>();
                result.push(skeleton.clone());
            } else {
                result.push((*word).to_string());
            }
        }

        result.join(".")
    }

    fn report(&self, score: usize, domain_original: &str, domain: &str, punycode_detected: bool) {
        if score >= 90 && punycode_detected {
            println!(
                "Homoglyph detected {} (Punycode: {})",
                style(domain).red().on_black().bold(),
                domain_original
            );
        } else if score >= 90 {
            println!("Suspicious {} (score {})", style(domain).red(), score);
        } else if score >= 70 {
            println!("Suspicious {} (score {})", style(domain).yellow(), score);
        } else if score >= 56 {
            println!(
                "Suspicious {} (score {})",
                style(domain_original).magenta(),
                score
            );
        }

        if score >= 56 {
            if punycode_detected {
                info!("{} - (Punycode: {})", domain, domain_original);
            } else {
                info!("{}", domain);
            }
        }
    }
}

