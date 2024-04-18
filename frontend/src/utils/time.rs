use time::macros::format_description;

type FormatDescription<'a> = &'a [time::format_description::BorrowedFormatItem<'a>];

pub const DATETIME_FORMAT: FormatDescription =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
