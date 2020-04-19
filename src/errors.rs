use std;
use url;
use tungstenite;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {

    }

    foreign_links {
        IoError(std::io::Error);
        UrlParserError(url::ParseError);
        Tungstenite(tungstenite::Error);
    }
}
