use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout as TuiLayout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new(include_str!("hobbit-ch01.txt").into()).run(&mut terminal);
    ratatui::restore();
    result
}

struct Theme {
    text: Color,
    background: Color,
    highlight: Color,
    shadow: Color,
}

const THEME_KEY_BASE: Theme = Theme {
    text: Color::Rgb(16, 24, 48),
    background: Color::Rgb(48, 72, 144),
    highlight: Color::Rgb(64, 96, 192),
    shadow: Color::Rgb(32, 48, 96),
};

const THEME_KEY_HINT: Theme = Theme {
    text: Color::Rgb(16, 48, 16),
    background: Color::Rgb(48, 144, 48),
    highlight: Color::Rgb(64, 192, 64),
    shadow: Color::Rgb(32, 96, 32),
};

struct Key {
    theme: &'static Theme,
    text: Line<'static>,
}

impl Widget for &Key {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Set the style for the button
        buf.set_style(
            area,
            Style::new().bg(self.theme.background).fg(self.theme.text),
        );
        // Render the top line
        if area.height > 2 {
            buf.set_string(
                area.x,
                area.y,
                " ".repeat(area.width as usize),
                Style::new()
                    .fg(self.theme.highlight)
                    .bg(self.theme.background),
            )
        }
        // Render the bottom line
        if area.height > 1 {
            buf.set_string(
                area.x,
                area.y + area.height - 1,
                "▁".repeat(area.width as usize),
                Style::new().fg(self.theme.shadow).bg(self.theme.background),
            );
        }
        // Render the label
        let margin_x = area.width.saturating_sub(self.text.width() as u16) / 2;
        let margin_y = area.height.saturating_sub(1) / 2;
        buf.set_line(area.x + margin_x, area.y + margin_y, &self.text, area.width);
    }
}

pub struct Keyboard {
    layout: &'static Layout,
    keys: Vec<Vec<Key>>,
    draw: bool,
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard::from_layout(&LAYOUT_QWERTY)
    }
}

impl Keyboard {
    fn toggle_draw(&mut self) {
        self.draw = !self.draw;
    }
    fn next_layout(&mut self) {
        if std::ptr::eq(self.layout, &LAYOUT_QWERTY) {
            self.set_dvorak();
        } else if std::ptr::eq(self.layout, &LAYOUT_DVORAK) {
            self.set_3l();
        } else if std::ptr::eq(self.layout, &LAYOUT_3L) {
            self.set_qwerty();
        }
    }
    fn from_layout(layout: &'static Layout) -> Self {
        let mut keys = vec![];
        for row in layout.base {
            let mut row_keys = vec![];
            for key in *row {
                let text = if *key == '\0' {
                    Line::from("").centered()
                } else {
                    Line::from(key.to_string().bold().white()).centered()
                };
                row_keys.push(Key {
                    theme: &THEME_KEY_BASE,
                    text,
                })
            }
            keys.push(row_keys)
        }
        Self {
            keys,
            layout,
            draw: true,
        }
    }
    fn set_qwerty(&mut self) {
        *self = Self::from_layout(&LAYOUT_QWERTY)
    }
    fn set_dvorak(&mut self) {
        *self = Self::from_layout(&LAYOUT_DVORAK)
    }
    fn set_3l(&mut self) {
        *self = Self::from_layout(&LAYOUT_3L)
    }
}

impl Widget for &Keyboard {
    fn render(self, block_area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Render the surrounding block
        let title = Line::from(format!(" Layout - {} ", self.layout.name).bold());
        let instructions = Line::from(vec![
            " Toggle Hints ".into(),
            "<C-h> ".blue().bold(),
            " Next Layout ".into(),
            "<C-n> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .dark_gray()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED);
        let area = block.inner(block_area);
        block.render(block_area, buf);
        // Get the vertical layout for the keyboard
        let rows_num = self.layout.base.len();
        let mut row_height = area.height / rows_num as u16;
        if row_height % 2 == 0 {
            row_height -= 1;
        }
        let layout = {
            let mut constraints = vec![];
            constraints.push(Constraint::Fill(1));
            constraints.extend(std::iter::repeat_n(
                Constraint::Length(row_height),
                rows_num,
            ));
            constraints.push(Constraint::Fill(1));
            TuiLayout::vertical(constraints).split(area)
        };
        // Render the rows
        let cols_max = self.layout.base.iter().map(|row| row.len()).max().unwrap();
        let mut col_width = area.width / cols_max as u16;
        if col_width % 2 == 0 {
            col_width -= 1;
        }
        for (row_area, row) in layout.iter().skip(1).take(rows_num).zip(&self.keys) {
            let key_layout = {
                let mut constraints = vec![];
                constraints.push(Constraint::Fill(1));
                constraints.extend(std::iter::repeat_n(Constraint::Length(col_width), cols_max));
                constraints.push(Constraint::Fill(1));
                TuiLayout::horizontal(constraints).split(*row_area)
            };
            for (key_area, key) in key_layout.iter().skip(1).take(cols_max).zip(row) {
                key.render(*key_area, buf);
            }
        }
    }
}

pub struct App {
    keyboard: Keyboard,
    story: String,
    progress: usize,
    exit: bool,
}

impl App {
    pub fn next(&self) -> Option<char> {
        self.story.chars().nth(self.progress)
    }
    pub fn new(story: String) -> Self {
        Self {
            keyboard: Keyboard::default(),
            story: story
                .replace("\n", "↩")
                .replace("—", "-")
                .replace("—", "-")
                .replace("’", "'"),
            progress: 0,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        if self.keyboard.draw {
            let vertical = TuiLayout::vertical([Constraint::Fill(2), Constraint::Fill(1)]);
            let [app, keyboard] = vertical.areas(frame.area());
            frame.render_widget(&self.keyboard, keyboard);
            frame.render_widget(self, app);
        } else {
            frame.render_widget(self, frame.area());
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => self.exit(),
            KeyEvent {
                code: KeyCode::Char('n'),
                modifiers,
                ..
            } if modifiers.contains(KeyModifiers::CONTROL) => self.keyboard.next_layout(),
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers,
                ..
            } if modifiers.contains(KeyModifiers::CONTROL) => self.keyboard.toggle_draw(),
            KeyEvent {
                code: KeyCode::Char(char),
                ..
            } => {
                if self.next() == Some(char) {
                    self.progress += 1;
                }
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                if self.next() == Some('↩') {
                    self.progress += 1;
                }
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, block_area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Story ".bold());
        let instructions = Line::from(vec![" Exit ".into(), "<Esc> ".blue().bold()]);
        let block = Block::bordered()
            .dark_gray()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED);
        let buff_width = block_area.width as usize / 3;
        let mut story = self
            .story
            .chars()
            .skip(self.progress.saturating_sub(buff_width));
        let prefix_len = self.progress - self.progress.saturating_sub(buff_width);
        let prefix = (&mut story).take(prefix_len).collect::<String>();
        let current = (&mut story).take(1).collect::<String>();
        let postfix_len = (2 * buff_width).saturating_sub(prefix_len);
        let postfix = story.take(postfix_len).collect::<String>();
        let counter_text = Text::from(vec![Line::from(vec![
            prefix.dark_gray(),
            current.white().bold(),
            postfix.gray(),
        ])]);
        let area = block.inner(block_area);
        block.render(block_area, buf);
        let [_, area, _] = TuiLayout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);
        Paragraph::new(counter_text).centered().render(area, buf);
    }
}

type Layer = &'static [&'static [char]];

enum Modifier {
    Shift,
    Sym,
    Cur,
}

struct Location {
    row: u8,
    col: u8,
    modifier: Option<Modifier>,
}

struct Layout {
    name: &'static str,
    base: Layer,
    sym: Layer,
    cur: Layer,
}

impl Layout {
    fn shift(c: char) -> char {
        match c {
            '`' => '~',
            '1' => '!',
            '2' => '@',
            '3' => '#',
            '4' => '$',
            '5' => '%',
            '6' => '^',
            '7' => '&',
            '8' => '*',
            '9' => '(',
            '0' => ')',
            '[' => '{',
            ']' => '}',
            '\'' => '"',
            ',' => '<',
            '.' => '>',
            '/' => '?',
            '=' => '+',
            '\\' => '|',
            '-' => '_',
            ';' => ':',
            c => c.to_ascii_uppercase(),
        }
    }
    fn location(&self, c: char) -> Option<Location> {
        // Check the base layer
        for (row_i, row) in self.base.iter().enumerate() {
            for (col_i, c_candidate) in row.iter().enumerate() {
                if *c_candidate == c {
                    return Some(Location {
                        row: row_i as u8,
                        col: col_i as u8,
                        modifier: None,
                    });
                }
            }
        }
        // Check the sym layer
        for (row_i, row) in self.sym.iter().enumerate() {
            for (col_i, c_candidate) in row.iter().enumerate() {
                if *c_candidate == c {
                    return Some(Location {
                        row: row_i as u8,
                        col: col_i as u8,
                        modifier: Some(Modifier::Sym),
                    });
                }
            }
        }
        // Check the cur layer
        for (row_i, row) in self.cur.iter().enumerate() {
            for (col_i, c_candidate) in row.iter().enumerate() {
                if *c_candidate == c {
                    return Some(Location {
                        row: row_i as u8,
                        col: col_i as u8 + 6,
                        modifier: Some(Modifier::Cur),
                    });
                }
            }
        }
        // Check the shifted base layer
        for (row_i, row) in self.base.iter().enumerate() {
            for (col_i, c_candidate) in row.iter().enumerate() {
                if *c_candidate == c.to_ascii_uppercase() {
                    return Some(Location {
                        row: row_i as u8,
                        col: col_i as u8,
                        modifier: Some(Modifier::Shift),
                    });
                }
            }
        }
        None
    }
}

const LAYOUT_QWERTY: Layout = Layout {
    name: "QWERTY",
    base: KEYS_QWERTY_BASE,
    sym: &[],
    cur: &[],
};

const KEYS_QWERTY_BASE: &[&[char]] = &[
    &[
        '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '[', ']', '\0',
    ],
    &[
        '\0', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\\',
    ],
    &[
        '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '\0', '\0',
    ],
    &[
        '\0', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', '\0', '\0', '\0',
    ],
];

const LAYOUT_DVORAK: Layout = Layout {
    name: "Dvorak",
    base: KEYS_DVORAK_BASE,
    sym: &[],
    cur: &[],
};

const KEYS_DVORAK_BASE: &[&[char]] = &[
    &[
        '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '[', ']', '\0',
    ],
    &[
        '\0', '\'', ',', '.', 'p', 'y', 'f', 'g', 'c', 'r', '/', '=', '\\', '\0',
    ],
    &[
        '\0', 'a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's', '-', '\0', '\0',
    ],
    &[
        '\0', ';', 'q', 'j', 'k', 'x', 'b', 'm', 'w', 'v', 'z', '\0', '\0', '\0',
    ],
];

const LAYOUT_3L: Layout = Layout {
    name: "3l",
    base: KEYS_3L_BASE,
    sym: KEYS_3L_SYM,
    cur: KEYS_3L_CUR,
};

const KEYS_3L_BASE: &[&[char]] = &[
    &['q', 'f', 'u', 'y', 'z', 'x', 'k', 'c', 'w', 'b'],
    &['o', 'h', 'e', 'a', 'i', 'd', 'r', 't', 'n', 's'],
    &[',', 'm', '.', 'j', ';', 'g', 'l', 'p', 'v', '\0'],
];

const KEYS_3L_SYM: &[&[char]] = &[
    &['"', '_', '[', ']', '^', '!', '<', '>', '=', '&'],
    &['/', '-', '{', '}', '*', '?', '(', ')', '\'', ':'],
    &['#', '$', '|', '~', '`', '+', '%', '\\', '@'],
];
const KEYS_3L_CUR: &[&[char]] = &[
    &['\0', '1', '2', '3'],
    &['\0', '4', '5', '6'],
    &['0', '7', '8', '9'],
];
