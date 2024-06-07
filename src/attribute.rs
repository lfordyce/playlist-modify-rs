use std::iter::FusedIterator;

#[derive(Clone, Debug)]
pub(crate) struct AttributePairs<'a> {
    string: &'a str,
    index: usize,
}

impl<'a> AttributePairs<'a> {
    pub const fn new(string: &'a str) -> Self { Self { string, index: 0 } }
}

impl<'a> Iterator for AttributePairs<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.string.as_bytes().get(self.index + 1)?;

        let key = {
            // the position in the string:
            let start = self.index;
            // the key ends at an `=`:
            let end = self.string[self.index..]
                .char_indices()
                .find_map(|(i, c)| if c == '=' { Some(i) } else { None })?
                + self.index;

            // advance the index to the char after the end of the key (to skip the `=`)
            // NOTE: it is okay to add 1 to the index, because an `=` is exactly 1 byte.
            self.index = end + 1;
            self.string[start..end].trim()
        };

        let value = {
            let start = self.index;

            // find the end of the value by searching for `,`.
            // it should ignore `,` that are inside double quotes.
            let mut inside_quotes = false;

            let end = {
                let mut result = self.string.len();

                for (i, c) in self.string[self.index..].char_indices() {
                    // if a quote is encountered
                    if c == '"' {
                        // update variable
                        inside_quotes = !inside_quotes;
                        // terminate if a comma is encountered, which is not in a
                        // quote
                    } else if c == ',' && !inside_quotes {
                        // move the index past the comma
                        self.index += 1;
                        // the result is the index of the comma (comma is not included in the
                        // resulting string)
                        result = i + self.index - 1;
                        break;
                    }
                }

                result
            };

            self.index += end;
            self.index -= start;
            self.string[start..end].trim()
        };

        Some((key, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut remaining = 0;

        // each `=` in the remaining str is an iteration
        // this also ignores `=` inside quotes!
        let mut inside_quotes = false;

        for (_, c) in self.string[self.index..].char_indices() {
            if c == '=' && !inside_quotes {
                remaining += 1;
            } else if c == '"' {
                inside_quotes = !inside_quotes;
            }
        }

        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for AttributePairs<'a> {}
impl<'a> FusedIterator for AttributePairs<'a> {}