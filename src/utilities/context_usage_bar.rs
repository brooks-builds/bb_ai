use colored::Colorize;

pub fn context_usage_bar(
    tokens_used: u32,
    max_tokens: u32,
    bar_length: u32,
) -> colored::ColoredString {
    let mut bar = String::new();
    let progress_char = '=';
    let token_used_percentage = tokens_used / max_tokens;
    let mut bar_chars = token_used_percentage * bar_length;
    let color_change = bar_length / 3;
    let bars_used = bar_chars;

    while bar_chars > 0 {
        bar.push(progress_char);
        bar_chars -= 1
    }

    bar.push('>');

    for _ in bar.len()..bar_length as usize {
        bar.push(' ');
    }

    if bars_used < color_change {
        bar.green()
    } else if bars_used > bars_used - color_change {
        bar.red()
    } else {
        bar.yellow()
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn contect_usage_bar_shows_percentage_used() {
        let tokens_used = 500;
        let max_tokens = 200000;
        let bar_length = 10;
        let expected = ">         ".to_owned();

        assert_eq!(
            context_usage_bar(tokens_used, max_tokens, bar_length),
            expected.green()
        );
    }

    #[test]
    fn contect_usage_bar_shows_green_on_low_context() {
        let tokens_used = 500;
        let max_tokens = 200000;
        let bar_length = 10;
        let expected = ">         ".to_owned();

        assert_eq!(
            context_usage_bar(tokens_used, max_tokens, bar_length),
            expected.green()
        );
    }
}
