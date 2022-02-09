pub fn utc_str_to_china_str(utc_str: &str) -> String {
    let utc_str = format!("{} +00:00:00", utc_str);
    let format1 = time::format_description::parse(
      "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]",
  )
  .unwrap();
    let format2 =
        time::format_description::parse("[year]年[month]月[day]日[hour]时[minute]分[second]秒")
            .unwrap();
    let china_offset = time::UtcOffset::from_hms(8, 0, 0).unwrap();
    let time = time::OffsetDateTime::parse(utc_str.as_str(), &format1).unwrap();
    return time.to_offset(china_offset).format(&format2).unwrap();
}

pub fn generate_timestamp() -> String {
    let china_offset = time::UtcOffset::from_hms(8, 0, 0).unwrap();
    let format =
        time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
    return time::OffsetDateTime::now_utc()
        .to_offset(china_offset)
        .format(&format)
        .unwrap();
}
