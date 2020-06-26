use serde::{Serialize, Deserialize};
use woothee::parser::Parser;

// This is basically a dumb to_owned copy of the fields
// But that's ok, I'd rather have these thin wrappers
// with a coherent interface.
#[derive(Serialize, Deserialize, Debug)]
pub struct UserAgent {
  pub name: String,
  pub platform: String,
  pub os: String,
  pub os_version: String,
  pub browser_type: String,
  pub version: String,
  pub vendor: String,
}

impl Default for UserAgent {
  fn default() -> UserAgent {
    UserAgent {
      name: "UNKNOWN".to_owned(),
      platform: "UNKNOWN".to_owned(),
      os: "UNKNOWN".to_owned(),
      os_version: "UNKNOWN".to_owned(),
      browser_type: "UNKNOWN".to_owned(),
      version: "UNKNOWN".to_owned(),
      vendor: "UNKNOWN".to_owned(),
    }
  }
}

impl From<&str> for UserAgent {
  fn from(ua: &str) -> UserAgent {
    let result = Parser::new().parse(ua);

    match result {
      None => UserAgent::default(),
      Some(ua) => UserAgent {
        name: ua.name.to_owned(),
        platform: ua.category.to_owned(),
        os: ua.os.to_owned(),
        os_version: ua.os_version.to_string(),
        browser_type: ua.browser_type.to_owned(),
        version: ua.version.to_owned(),
        vendor: ua.vendor.to_owned(),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_useragent_from_string() {
    let ua_string = "Mozilla/4.0 (compatible; MSIE 8.0; Windows NT 6.1; Trident/4.0)";
    let user_agent = UserAgent::from(ua_string);

    assert_eq!(user_agent.name, "Internet Explorer".to_owned());
  }
  
  #[test]
  fn test_useragent_from_string_empty() {
    let ua_string = "";
    let user_agent = UserAgent::from(ua_string);

    assert_eq!(user_agent.platform, "UNKNOWN".to_owned());
  }
}
