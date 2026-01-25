use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Skill 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub enabled_tools: Vec<String>,
}

/// Skill 管理器
pub struct SkillManager {
    skills: HashMap<String, Skill>,
    active_skill: Option<String>,
    config_path: PathBuf,
}

impl SkillManager {
    pub fn new() -> Self {
        let config_path = PathBuf::from(".kota_skills.json");
        let mut manager = Self {
            skills: HashMap::new(),
            active_skill: None,
            config_path,
        };

        // 加载默认技能
        manager.load_default_skills();

        // 尝试从文件加载自定义技能
        let _ = manager.load_from_file();

        manager
    }

    /// 加载默认技能
    fn load_default_skills(&mut self) {
        // 代码审查技能
        self.add_skill(Skill {
            name: "code_review".to_string(),
            description: "专注于代码审查和质量分析".to_string(),
            prompt: "你是一个专业的代码审查专家。请仔细检查代码的质量、安全性、性能和最佳实践。"
                .to_string(),
            enabled_tools: vec![
                "read_file".to_string(),
                "scan_codebase".to_string(),
                "grep_search".to_string(),
            ],
        });

        // 重构技能
        self.add_skill(Skill {
            name: "refactor".to_string(),
            description: "专注于代码重构和优化".to_string(),
            prompt: "你是一个代码重构专家。帮助改进代码结构、可读性和可维护性，同时保持功能不变。"
                .to_string(),
            enabled_tools: vec![
                "read_file".to_string(),
                "edit_file".to_string(),
                "write_file".to_string(),
            ],
        });

        // 调试技能
        self.add_skill(Skill {
            name: "debug".to_string(),
            description: "专注于问题诊断和调试".to_string(),
            prompt: "你是一个调试专家。帮助定位和修复代码中的问题，提供详细的分析和解决方案。"
                .to_string(),
            enabled_tools: vec![
                "read_file".to_string(),
                "execute_bash".to_string(),
                "grep_search".to_string(),
            ],
        });

        // 文档技能
        self.add_skill(Skill {
            name: "documentation".to_string(),
            description: "专注于编写和改进文档".to_string(),
            prompt: "你是一个技术文档专家。帮助创建清晰、准确、易懂的文档和注释。".to_string(),
            enabled_tools: vec![
                "read_file".to_string(),
                "write_file".to_string(),
                "scan_codebase".to_string(),
            ],
        });
    }

    /// 添加技能
    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// 获取技能
    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// 列出所有技能
    pub fn list_skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    /// 激活技能
    pub fn activate_skill(&mut self, name: &str) -> Result<()> {
        if self.skills.contains_key(name) {
            self.active_skill = Some(name.to_string());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Skill '{}' not found", name))
        }
    }

    /// 停用技能
    pub fn deactivate_skill(&mut self) {
        self.active_skill = None;
    }

    /// 获取当前激活的技能
    pub fn get_active_skill(&self) -> Option<&Skill> {
        self.active_skill
            .as_ref()
            .and_then(|name| self.skills.get(name))
    }

    /// 获取增强的 preamble
    pub fn get_enhanced_preamble(&self, base_preamble: &str) -> String {
        if let Some(skill) = self.get_active_skill() {
            format!(
                "{}\n\n[ACTIVE SKILL: {}]\n{}\n\n可用工具: {}",
                base_preamble,
                skill.name,
                skill.prompt,
                skill.enabled_tools.join(", ")
            )
        } else {
            base_preamble.to_string()
        }
    }

    /// 检查工具是否在当前技能中启用
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        if let Some(skill) = self.get_active_skill() {
            skill.enabled_tools.contains(&tool_name.to_string())
        } else {
            true // 如果没有激活技能，所有工具都可用
        }
    }

    /// 保存到文件
    pub fn save_to_file(&self) -> Result<()> {
        let skills_vec: Vec<&Skill> = self.skills.values().collect();
        let json = serde_json::to_string_pretty(&skills_vec)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// 从文件加载
    pub fn load_from_file(&mut self) -> Result<()> {
        if !self.config_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let skills: Vec<Skill> = serde_json::from_str(&content)?;

        for skill in skills {
            self.skills.insert(skill.name.clone(), skill);
        }

        Ok(())
    }

    /// 删除技能
    pub fn remove_skill(&mut self, name: &str) -> Result<()> {
        if self.skills.remove(name).is_some() {
            if self.active_skill.as_ref() == Some(&name.to_string()) {
                self.active_skill = None;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Skill '{}' not found", name))
        }
    }
}
