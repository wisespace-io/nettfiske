use std::collections::HashMap;

lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        
        // Crypto Exchanges
        m.insert("binance", 10);
        m.insert("bitfinex", 10);
        m.insert("coinbase", 10);
        m.insert("qryptos", 10);
        m.insert("huobi", 10);
        m.insert("bittrex", 10);
        m.insert("cobinhood", 10);   
        m.insert("bithumb", 10);     
        m.insert("bitpanda", 10); 
        m.insert("upbit", 10);
        m.insert("bit-z", 10);
        
        // Crypto wallets
        m.insert("myetherwallet", 10);
    
        // Crypto coins
        m.insert("iota", 10);
        m.insert("ripple", 10);
        m.insert("omisego", 10);
        m.insert("ethereum", 10);

        // Social Media
        m.insert("twitter", 10);
        m.insert("facebook", 10);
        m.insert("linkedin", 10);
        m.insert("instagram", 10);
        m.insert("telegram", 10);     
        m.insert("skype", 10);  
        m.insert("tinder", 10); 
        m.insert("snapchat", 10); 
        m.insert("whatsapp", 10);

        // Streaming
        m.insert("spotify", 10);
        m.insert("youtube", 10);
        m.insert("itunes", 10);
        m.insert("netflix", 10);

        // Payments/Banks
        m.insert("paypal", 10);
        m.insert("transferwise", 10);
        m.insert("westernunion", 10);
        m.insert("santander", 10);
        m.insert("hsbc", 10);

        // Emails        
        m.insert("gmail", 10);
        m.insert("office365", 10);        
        m.insert("yahoo", 10);
        m.insert("google", 10);
        m.insert("outlook", 10);

        // Ecommerce        
        m.insert("amazon", 10);
        m.insert("overstock", 10);        
        m.insert("alibaba", 10);
        m.insert("aliexpress", 10);
        m.insert("zalando", 10);
        m.insert("rakuten", 10);
        m.insert("groupon", 10);        
        m.insert("blocket", 10);
        m.insert("expedia", 10);
        m.insert("baidu", 10);

        // Commons
        m.insert("microsoft", 10);
        m.insert("apple", 10);
        m.insert("appleid", 10);
        m.insert("icloud", 10);

        // Utils
        m.insert("dropbox", 10);
        m.insert("lastpass", 10);
        m
    };
}
