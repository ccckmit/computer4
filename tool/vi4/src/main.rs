use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
// 修正 1：引入標準庫的 Result 取代 crossterm::Result
use std::io::{stdout, Result, Write};

/// 編輯器的三種基本模式
#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

struct Editor {
    cx: usize,
    cy: usize,
    mode: Mode,
    lines: Vec<Vec<char>>,
    should_quit: bool,
    command_buffer: String,
}

impl Editor {
    fn new() -> Self {
        Self {
            cx: 0,
            cy: 0,
            mode: Mode::Normal,
            lines: vec![Vec::new()],
            should_quit: false,
            command_buffer: String::new(),
        }
    }

    fn run(&mut self) -> Result<()> {
        while !self.should_quit {
            self.draw_screen()?;
            self.process_keypress()?;
        }
        Ok(())
    }

    fn draw_screen(&self) -> Result<()> {
        let mut stdout = stdout();
        let (_cols, rows) = size()?;
        
        queue!(stdout, Hide, MoveTo(0, 0))?;

        // 繪製文字內容
        for y in 0..(rows - 1) {
            queue!(stdout, Clear(ClearType::CurrentLine))?;
            // 修正 2：將 y as usize 用括號包起來，避免編譯器誤認為是泛型括號 < >
            if (y as usize) < self.lines.len() {
                let line: String = self.lines[y as usize].iter().collect();
                queue!(stdout, Print(line))?;
            } else {
                queue!(stdout, Print("~"))?;
            }
            queue!(stdout, Print("\r\n"))?;
        }

        // 繪製狀態列 / 命令列
        queue!(stdout, Clear(ClearType::CurrentLine))?;
        queue!(
            stdout,
            SetBackgroundColor(Color::White),
            SetForegroundColor(Color::Black)
        )?;
        
        match self.mode {
            Mode::Normal => queue!(stdout, Print(" NORMAL "))?,
            Mode::Insert => queue!(stdout, Print(" INSERT "))?,
            Mode::Command => queue!(stdout, Print(format!(" :{} ", self.command_buffer)))?,
        }
        
        queue!(stdout, ResetColor)?;

        // 放置游標
        if self.mode == Mode::Command {
            queue!(stdout, MoveTo((self.command_buffer.len() + 2) as u16, rows - 1))?;
        } else {
            queue!(stdout, MoveTo(self.cx as u16, self.cy as u16))?;
        }

        queue!(stdout, Show)?;
        stdout.flush()?;
        Ok(())
    }

    fn process_keypress(&mut self) -> Result<()> {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = read()? {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                self.should_quit = true;
                return Ok(());
            }

            match self.mode {
                Mode::Normal => self.process_normal(code),
                Mode::Insert => self.process_insert(code),
                Mode::Command => self.process_command(code),
            }
        }
        Ok(())
    }

    fn process_normal(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('h') | KeyCode::Left => self.cx = self.cx.saturating_sub(1),
            KeyCode::Char('j') | KeyCode::Down => {
                if self.cy < self.lines.len() - 1 {
                    self.cy += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => self.cy = self.cy.saturating_sub(1),
            KeyCode::Char('l') | KeyCode::Right => self.cx += 1,
            
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            }
            _ => {}
        }
        self.fix_cursor();
    }

    fn process_insert(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.cx = self.cx.saturating_sub(1);
            }
            KeyCode::Left => self.cx = self.cx.saturating_sub(1),
            KeyCode::Right => self.cx += 1,
            KeyCode::Up => self.cy = self.cy.saturating_sub(1),
            KeyCode::Down => {
                if self.cy < self.lines.len() - 1 {
                    self.cy += 1;
                }
            }
            KeyCode::Char(c) => {
                self.lines[self.cy].insert(self.cx, c);
                self.cx += 1;
            }
            KeyCode::Enter => {
                let rest: Vec<char> = self.lines[self.cy].drain(self.cx..).collect();
                self.lines.insert(self.cy + 1, rest);
                self.cy += 1;
                self.cx = 0;
            }
            KeyCode::Backspace => {
                if self.cx > 0 {
                    self.cx -= 1;
                    self.lines[self.cy].remove(self.cx);
                } else if self.cy > 0 {
                    let current_line = self.lines.remove(self.cy);
                    self.cy -= 1;
                    self.cx = self.lines[self.cy].len();
                    self.lines[self.cy].extend(current_line);
                }
            }
            _ => {}
        }
        self.fix_cursor();
    }

    fn process_command(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => {
                if self.command_buffer == "q" {
                    self.should_quit = true;
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char(c) => self.command_buffer.push(c),
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            _ => {}
        }
    }

    fn fix_cursor(&mut self) {
        if self.cy >= self.lines.len() {
            self.cy = self.lines.len().saturating_sub(1);
        }
        let line_len = self.lines[self.cy].len();
        
        if self.cx > line_len {
            self.cx = line_len;
        }
        
        if self.mode == Mode::Normal && self.cx == line_len && line_len > 0 {
            self.cx = line_len - 1;
        }
    }
}

struct Cleanup;
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    
    let _cleanup = Cleanup;

    let mut editor = Editor::new();
    editor.run()?;

    Ok(())
}