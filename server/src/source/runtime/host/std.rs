use chrono::{Local, NaiveDateTime, Utc};
use chrono_tz::Tz;
use wasmer::FunctionEnvMut;

use super::HostState;
use crate::source::runtime::store::DescriptorValue;

const INVALID_DESCRIPTOR: i32 = -1;
const FAILED_MEMORY_WRITE: i32 = -3;
const INVALID_STRING: i32 = -4;
const INVALID_DATE_STRING: i32 = -5;

pub(super) fn destroy(mut env: FunctionEnvMut<HostState>, descriptor: i32) {
    env.data_mut().descriptors.remove(descriptor);
}

pub(super) fn buffer_len(env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    env.data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_bytes)
        .and_then(|bytes| i32::try_from(bytes.len()).ok())
        .unwrap_or(INVALID_DESCRIPTOR)
}

pub(super) fn read_buffer(
    env: FunctionEnvMut<HostState>,
    descriptor: i32,
    pointer: i32,
    size: i32,
) -> i32 {
    if pointer < 0 || size < 0 {
        return FAILED_MEMORY_WRITE;
    }
    let Some(bytes) = env
        .data()
        .descriptors
        .get(descriptor)
        .and_then(DescriptorValue::as_bytes)
    else {
        return INVALID_DESCRIPTOR;
    };
    if size as usize > bytes.len() {
        return FAILED_MEMORY_WRITE;
    }
    env.data()
        .write_bytes(&env, pointer as u32, &bytes[..size as usize])
        .map(|_| 0)
        .unwrap_or(FAILED_MEMORY_WRITE)
}

pub(super) fn current_date(_env: FunctionEnvMut<HostState>) -> f64 {
    Utc::now().timestamp() as f64
}

pub(super) fn utc_offset(_env: FunctionEnvMut<HostState>) -> i64 {
    Local::now().offset().local_minus_utc() as i64
}

#[allow(clippy::too_many_arguments)]
pub(super) fn parse_date(
    env: FunctionEnvMut<HostState>,
    date_pointer: i32,
    date_length: i32,
    format_pointer: i32,
    format_length: i32,
    _locale_pointer: i32,
    _locale_length: i32,
    timezone_pointer: i32,
    timezone_length: i32,
) -> f64 {
    let Ok(date) = env.data().read_string(&env, date_pointer, date_length) else {
        return INVALID_STRING as f64;
    };
    let Ok(format) = env.data().read_string(&env, format_pointer, format_length) else {
        return INVALID_STRING as f64;
    };
    let timezone = if timezone_length > 0 {
        env.data()
            .read_string(&env, timezone_pointer, timezone_length)
            .ok()
    } else {
        None
    };
    let format = swift_date_format_to_chrono(&format);
    let Some(parsed) = NaiveDateTime::parse_from_str(&date, &format).ok() else {
        return INVALID_DATE_STRING as f64;
    };
    if timezone.as_deref() == Some("current") {
        return parsed
            .and_local_timezone(*Local::now().offset())
            .single()
            .map(|value| value.timestamp() as f64)
            .unwrap_or(INVALID_DATE_STRING as f64);
    }
    let zone = timezone
        .as_deref()
        .and_then(|value| value.parse::<Tz>().ok())
        .unwrap_or(Tz::UTC);
    parsed
        .and_local_timezone(zone)
        .single()
        .map(|value| value.timestamp() as f64)
        .unwrap_or(INVALID_DATE_STRING as f64)
}

fn swift_date_format_to_chrono(format: &str) -> String {
    let mut result = String::new();
    let mut chars = format.chars().peekable();
    while let Some(character) = chars.next() {
        let token = match character {
            'y' => {
                let count = consume(&mut chars, character);
                if count == 2 {
                    "%y"
                } else {
                    "%Y"
                }
            }
            'M' => match consume(&mut chars, character) {
                1 | 2 => "%m",
                3 => "%b",
                _ => "%B",
            },
            'd' => {
                consume(&mut chars, character);
                "%d"
            }
            'H' => {
                consume(&mut chars, character);
                "%H"
            }
            'h' => {
                consume(&mut chars, character);
                "%I"
            }
            'm' => {
                consume(&mut chars, character);
                "%M"
            }
            's' => {
                consume(&mut chars, character);
                "%S"
            }
            'a' => "%p",
            'E' => {
                if consume(&mut chars, character) >= 4 {
                    "%A"
                } else {
                    "%a"
                }
            }
            'z' => {
                if consume(&mut chars, character) >= 4 {
                    "%Z"
                } else {
                    "%z"
                }
            }
            'Z' => {
                consume(&mut chars, character);
                "%Z"
            }
            other => {
                result.push(other);
                continue;
            }
        };
        result.push_str(token);
    }
    result
}

fn consume(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, character: char) -> usize {
    let mut count = 1;
    while chars.peek() == Some(&character) {
        chars.next();
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::swift_date_format_to_chrono;

    #[test]
    fn converts_common_source_date_formats() {
        assert_eq!(
            swift_date_format_to_chrono("yyyy-MM-dd HH:mm:ss"),
            "%Y-%m-%d %H:%M:%S"
        );
        assert_eq!(swift_date_format_to_chrono("MMM d, yyyy"), "%b %d, %Y");
    }
}
