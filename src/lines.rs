#[derive(Debug, Clone, Copy)]
pub struct EditorPosition {
    pub line: usize,
    pub column: usize,
}

pub fn find_position(initial_contents: &str, byte_offset: usize) -> EditorPosition {
    let mut contents: &str = &initial_contents.clone();
    let mut vec_lines = vec![];
    // line index, first byte index
    vec_lines.push((0, 0));
    let contents_len = contents.len();
    let mut newline_char_offset = contents.find("\n").unwrap_or(contents_len);
    let (line, line_start) = if newline_char_offset == contents_len {
        // we are on last line
        let last = vec_lines.last().unwrap();
        (last.0 + 1, last.1)
    } else {
        while byte_offset >= vec_lines.last().unwrap().1 {
            let next_index = vec_lines.last().unwrap().0 + 1;
            let next_line_offset = vec_lines.last().unwrap().1 + newline_char_offset + 1;
            if next_line_offset >= initial_contents.len() {
                break;
            }
            vec_lines.push((next_index, next_line_offset));
            contents = &initial_contents[next_line_offset..];
            newline_char_offset = contents.find("\n").unwrap_or(contents.len());
        }
        if byte_offset >= vec_lines.last().unwrap().1 {
            let last = vec_lines.last().unwrap();
            (last.0 + 1, last.1)
        } else {
            let prev_before_last = vec_lines[vec_lines.len() - 2];
            (prev_before_last.0 + 1, prev_before_last.1)
        }
    };

    let count = initial_contents[line_start..byte_offset].chars().count();
    let column = count + 1;

    EditorPosition { line, column }
}
