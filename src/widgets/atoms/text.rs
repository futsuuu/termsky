use ratatui::prelude::*;
#[cfg(test)]
use rstest::*;

use crate::widgets::{Store, Storeable};

pub struct Text<'a> {
    spans: Vec<Span<'a>>,
    alignment: Option<Alignment>,
    ignore_if_empty: bool,
}

impl<'a> Default for Text<'a> {
    fn default() -> Self {
        Self {
            spans: Vec::new(),
            alignment: None,
            ignore_if_empty: true,
        }
    }
}

const ELLIPSIS: char = '…';

impl<'a, S: Into<Span<'a>>> From<S> for Text<'a> {
    fn from(value: S) -> Self {
        Self {
            spans: vec![value.into()],
            ..Default::default()
        }
    }
}

impl<'a, S: Into<Span<'a>>> FromIterator<S> for Text<'a> {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        Self {
            spans: iter.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl<'a> Text<'a> {
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    pub fn ignore_if_empty(mut self, value: bool) -> Self {
        self.ignore_if_empty = value;
        self
    }

    fn lines<'b>(&self, width: usize, max_height: usize) -> Vec<Line<'a>> {
        if max_height == 0 {
            return Vec::with_capacity(0);
        }
        let mut lines: Vec<Line> = Vec::new();

        for span in &self.spans {
            let last_line = lines.last().map(|l| l.to_string()).unwrap_or_default();

            let content = format!("{last_line}{span}");
            let wrapped = wrap_and_restore_trailing_space(&content, width);
            for (i, s) in wrapped.into_iter().enumerate() {
                if i == 0 {
                    let Some(s) = s.strip_prefix(&last_line) else {
                        continue;
                    };
                    let span = Span::styled(s.to_string(), span.style);
                    lines = push_span(lines, span, false);
                } else if lines.len() < max_height {
                    let span = Span::styled(s.to_string(), span.style);
                    lines = push_span(lines, span, true);
                } else {
                    return set_ellipsis(trim_end(lines));
                }
            }
        }

        trim_end(lines)
    }
}

fn wrap_and_restore_trailing_space(content: &str, width: usize) -> Vec<std::borrow::Cow<'_, str>> {
    let mut wrapped = textwrap::wrap(content, width);
    if let Some(s) = wrapped.last_mut() {
        let trimmed = trailing_space(content);
        *s = format!("{s}{trimmed}").into();
    }
    wrapped
}

fn trim_end(lines: Vec<Line>) -> Vec<Line> {
    lines
        .into_iter()
        .map(|mut line| {
            if let Some(span) = line.spans.last_mut() {
                span.content = span.content.trim_end().to_string().into();
            }
            line
        })
        .collect()
}

fn push_span<'a>(mut lines: Vec<Line<'a>>, span: Span<'a>, new_line: bool) -> Vec<Line<'a>> {
    if new_line || lines.is_empty() {
        lines.push(Line::default());
    }
    let last_line = lines.last_mut().unwrap();
    last_line.spans.push(span);
    lines
}

fn set_ellipsis<'a>(mut lines: Vec<Line<'a>>) -> Vec<Line<'a>> {
    let Some(last_line) = lines.last_mut() else {
        return lines;
    };
    let Some(last_span) = last_line.spans.last_mut() else {
        return lines;
    };
    let mut s = last_span.content.to_string();
    s.pop();
    s.push(ELLIPSIS);
    last_span.content = s.into();
    return lines;
}

fn trailing_space<'a>(s: &'a str) -> &'a str {
    s.rsplit_once(|c: char| !c.is_whitespace() || c.is_control())
        .map(|(_, s)| s)
        .unwrap_or(s)
}

#[cfg(test)]
#[rstest]
#[case("asdf", "")]
#[case(" s     ", "     ")]
#[case("  ", "  ")]
#[case("\n   ", "   ")]
fn test_trailing_space(#[case] from: &str, #[case] to: &str) {
    assert_eq!(to, trailing_space(from));
}

impl<'a> Storeable<'a> for Text<'a> {
    fn store(self, area: Rect, store: &mut Store<'a>) {
        if area.is_empty() {
            return;
        }
        if self.ignore_if_empty && self.spans.iter().all(|s| s.width() == 0) {
            return;
        }
        let lines = self.lines(area.width as usize, area.height as usize);
        for (y, line) in lines.into_iter().enumerate() {
            let width = line.width() as u16;
            line.store(
                Rect {
                    x: area.x
                        + match self.alignment {
                            Some(Alignment::Left) | None => 0,
                            Some(Alignment::Center) => (area.width - width) / 2,
                            Some(Alignment::Right) => area.width - width,
                        },
                    y: area.y + y as u16,
                    width,
                    height: 1,
                },
                store,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::assert_buffer_eq;
    use rstest::*;

    use super::*;

    macro_rules! ellipsis {
        ($s:expr) => {
            format!("{}{}", $s, ELLIPSIS).as_str()
        };
    }

    #[test]
    fn test_set_ellipsis() {
        assert_eq!(
            vec![Line::from(ellipsis!("he"))],
            Text::from("hello").lines(3, 1)
        );
        assert_eq!(
            vec![
                Line::from("hello"),
                Line::from("hello"),
                Line::from(ellipsis!("hell")),
            ],
            Text::from_iter(["hello ", "hello ", "hello ", "hello"]).lines(7, 3)
        );
    }

    #[rstest]
    #[case::wrap_words(7, Text::from_iter(["hello ", "world"]), vec![
        Line::from("hello"),
        Line::from("world"),
    ])]
    #[case::wrap_chars(3, Text::from_iter(["hello ", "world"]), vec![
        Line::from("hel"),
        Line::from("lo"),
        Line::from("wor"),
        Line::from("ld"),
    ])]
    #[case::no_wrap(15, Text::from_iter(["hello ", "world"]), vec![
        Line::from_iter(["hello ", "world"]),
    ])]
    #[case::whitespace_span(15, Text::from_iter(["hello", " ", "  ", "world"]), vec![
        Line::from_iter(["hello", " ", "  ", "world"]),
    ])]
    #[case::newline(10, Text::from("a\nhello"), vec![
        Line::from("a"),
        Line::from("hello"),
    ])]
    #[case::newline_span(10, Text::from_iter(["a\n", "hello"]), vec![
        Line::from("a"),
        Line::from_iter(["", "hello"]),
    ])]
    fn wrap_spans(#[case] width: usize, #[case] text: Text, #[case] result: Vec<Line>) {
        assert_eq!(result, text.lines(width, usize::MAX));
    }

    #[test]
    fn one_word() {
        let mut store = Store::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));

        Text::from("hello").store(buf.area, &mut store);
        store.render(buf.area, &mut buf);
        assert_buffer_eq!(buf, Buffer::with_lines(vec!["hello     "]));
    }

    #[test]
    fn alignment() {
        let mut store = Store::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 11, 1));

        Text::from("hello")
            .alignment(Alignment::Center)
            .store(buf.area, &mut store);
        store.render(buf.area, &mut buf);
        assert_buffer_eq!(buf, Buffer::with_lines(vec!["   hello   "]));
    }
}
