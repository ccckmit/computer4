//! llm/agent.rs - AI Agent with memory and tool feedback
//! 
//! A simple agent that uses Ollama API with memory and tool execution capabilities.

use std::collections::VecDeque;
use std::process::Command;

pub const WORKSPACE: &str = "~/.agent0";
pub const MODEL: &str = "minimax-m2.5:cloud";
pub const MAX_TURNS: usize = 5;

pub const SYSTEM_PROMPT: &str = r#"你是 Jarvis，一個有用的 AI 助理。

重要規則：
1. 當你需要執行 shell 命令時，必須用 <shell> 標籤包住命令
2. <shell> 標籤內可以是多行命令（用反斜槓 \ 或 && 連接）
3. 當你完成所有操作後，用 <end/> 結束你的回覆

流程：
- 如果需要執行命令，輸出 <shell>...</shell>
- 執行完後我會顯示結果
- 如果還需要更多命令，繼續輸出 <shell>
- 當完成所有操作後，輸出 <end/> 表示結束"#;

pub struct Agent {
    conversation_history: VecDeque<String>,
    key_info: Vec<String>,
    workspace: String,
}

impl Agent {
    pub fn new() -> Self {
        Agent {
            conversation_history: VecDeque::new(),
            key_info: Vec::new(),
            workspace: WORKSPACE.to_string(),
        }
    }

    pub fn build_context(&self) -> String {
        let mut context_parts = Vec::new();
        
        if !self.key_info.is_empty() {
            let items: String = self.key_info.iter()
                .map(|k| format!("  <item>{}</item>", k))
                .collect::<Vec<_>>()
                .join("\n");
            context_parts.push(format!("<memory>\n{}\n</memory>", items));
        }
        
        if !self.conversation_history.is_empty() {
            let history: String = self.conversation_history.iter()
                .skip(self.conversation_history.len().saturating_sub(MAX_TURNS * 2))
                .cloned()
                .collect::<Vec<_>>()
                .join("\n");
            context_parts.push(format!("<history>\n{}\n</history>", history));
        }
        
        context_parts.join("\n\n")
    }

    pub fn update_memory(&mut self, user_input: &str, assistant_response: &str, tool_result: Option<&str>) {
        self.conversation_history.push_back(format!("  <user>{}</user>", user_input));
        self.conversation_history.push_back(format!("  <assistant>{}</assistant>", assistant_response));
        
        if let Some(result) = tool_result {
            let truncated = if result.len() > 500 { &result[..500] } else { result };
            self.conversation_history.push_back(format!("  <tool>{}</tool>", truncated));
        }
        
        while self.conversation_history.len() > MAX_TURNS * 4 {
            self.conversation_history.pop_front();
        }
    }

    pub fn add_key_info(&mut self, item: String) {
        if !item.is_empty() && !self.key_info.contains(&item) {
            self.key_info.push(item);
        }
    }

    #[allow(dead_code)]
    pub fn show_memory(&self) -> Vec<&String> {
        self.key_info.iter().collect()
    }

    #[allow(dead_code)]
    pub async fn call_ollama(&self, _prompt: &str, _system: &str) -> String {
        // Note: In real implementation, this would use reqwest for async HTTP
        // For now, returning a placeholder
        "Ollama API call requires async HTTP client (reqwest)".to_string()
    }

    pub fn execute_shell(&self, command: &str) -> Result<String, String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| format!("Execution error: {}", e))?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(format!("Error: {}", stderr))
        }
    }

    #[allow(dead_code)]
    pub fn parse_shell_tags(response: &str) -> Vec<String> {
        let mut cmds = Vec::new();
        
        // Use simple string search
        let tag = "<shell>";
        let end_tag = "</shell>";
        
        let mut remaining = response;
        while let Some(start_idx) = remaining.find(tag) {
            // Move past the <shell> tag
            remaining = &remaining[start_idx + tag.len()..];
            
            if let Some(end_idx) = remaining.find(end_tag) {
                let cmd = &remaining[..end_idx];
                cmds.push(cmd.trim().to_string());
                remaining = &remaining[end_idx + end_tag.len()..];
            } else {
                break;
            }
        }
        
        cmds
    }

    #[allow(dead_code)]
    pub fn has_end_tag(response: &str) -> bool {
        response.contains("<end/>")
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}