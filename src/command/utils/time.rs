use chrono::{NaiveDate, NaiveTime};

pub fn parse_time(time_str: &str) -> anyhow::Result<NaiveTime, chrono::format::ParseError> {
    NaiveTime::parse_from_str(time_str, "%H:%M:%S")
}

pub fn parse_date(date_str: &str) -> anyhow::Result<NaiveDate, chrono::format::ParseError> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
}

#[cfg(test)]
mod tests {
    use crate::command::utils::time::{parse_date, parse_time};

    use super::*;

    #[test]
    fn test_parse_time() {
        let time = parse_time("12:34:56").unwrap();
        assert_eq!(time, NaiveTime::from_hms_opt(12, 34, 56).unwrap());
    }

    #[test]
    fn test_parse_time_invalid() {
        let time = parse_time("12:34:56:78");
        assert!(time.is_err());
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("2021-01-01").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
    }
}
