use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Skill 定义 (SKILL.md 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub instructions: String, // Markdown body
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Skill 管理器
pub struct SkillManager {
    skills: HashMap<String, Skill>,
    active_skill: Option<String>,
    skills_dir: PathBuf,
}

impl SkillManager {
    pub fn new() -> Self {
        let skills_dir = PathBuf::from(".kota/skills");
        let mut manager = Self {
            skills: HashMap::new(),
            active_skill: None,
            skills_dir,
        };
        let _ = manager.load_skills();
        manager
    }

    /// 从 .kota/skills/ 目录加载所有 SKILL.md 文件
    fn load_skills(&mut self) -> Result<()> {
        if !self.skills_dir.exists() {
            fs::create_dir_all(&self.skills_dir)?;
            self.create_default_skills()?;
        }

        for entry in fs::read_dir(&self.skills_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    if let Ok(skill) = Self::parse_skill_md(&skill_file) {
                        self.skills.insert(skill.name.clone(), skill);
                    }
                }
            }
        }
        Ok(())
    }

    /// 解析 SKILL.md 文件
    fn parse_skill_md(path: &Path) -> Result<Skill> {
        let content = fs::read_to_string(path)?;
        let parts: Vec<&str> = content.splitn(3, "---").collect();

        if parts.len() < 3 {
            return Err(anyhow::anyhow!("Invalid SKILL.md format"));
        }

        let yaml = parts[1].trim();
        let instructions = parts[2].trim().to_string();

        let mut name = String::new();
        let mut description = String::new();
        let mut dependencies = Vec::new();

        for line in yaml.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                match key {
                    "name" => name = value.to_string(),
                    "description" => description = value.to_string(),
                    "dependencies" => dependencies.push(value.to_string()),
                    _ => {}
                }
            }
        }

        Ok(Skill {
            name,
            description,
            instructions,
            dependencies,
        })
    }

    /// 创建默认技能
    fn create_default_skills(&self) -> Result<()> {
        let defaults = vec![
            (
                "code-review",
                "Code review and quality analysis",
                "You are a professional code reviewer. Carefully check code quality, security, performance and best practices.",
            ),
            (
                "refactor",
                "Code refactoring and optimization",
                "You are a refactoring expert. Help improve code structure, readability and maintainability while preserving functionality.",
            ),
            (
                "debug",
                "Problem diagnosis and debugging",
                "You are a debugging expert. Help locate and fix issues in code with detailed analysis and solutions.",
            ),
        ];

        for (name, desc, inst) in defaults {
            let dir = self.skills_dir.join(name);
            fs::create_dir_all(&dir)?;
            let content = format!(
                "---\nname: {}\ndescription: {}\n---\n\n{}",
                name, desc, inst
            );
            fs::write(dir.join("SKILL.md"), content)?;
        }
        Ok(())
    }

    /// 添加技能
    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// 获取增强的 preamble
    pub fn get_enhanced_preamble(&self, base_preamble: &str) -> String {
        if let Some(skill) = self.get_active_skill() {
            format!(
                "{}\n\n[ACTIVE SKILL: {}]\n{}\n\n{}",
                base_preamble, skill.name, skill.description, skill.instructions
            )
        } else {
            base_preamble.to_string()
        }
    }

    /// 创建新技能
    pub fn create_skill(
        &mut self,
        name: &str,
        description: &str,
        instructions: &str,
    ) -> Result<()> {
        let dir = self.skills_dir.join(name);
        fs::create_dir_all(&dir)?;
        let content = format!(
            "---\nname: {}\ndescription: {}\n---\n\n{}",
            name, description, instructions
        );
        fs::write(dir.join("SKILL.md"), content)?;

        let skill = Skill {
            name: name.to_string(),
            description: description.to_string(),
            instructions: instructions.to_string(),
            dependencies: vec![],
        };
        self.skills.insert(name.to_string(), skill);
        Ok(())
    }

    /// 删除技能
    pub fn remove_skill(&mut self, name: &str) -> Result<()> {
        let dir = self.skills_dir.join(name);
        if dir.exists() {
            fs::remove_dir_all(dir)?;
        }
        if self.skills.remove(name).is_some() {
            if self.active_skill.as_ref() == Some(&name.to_string()) {
                self.active_skill = None;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Skill '{}' not found", name))
        }
    }

    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn list_skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    pub fn activate_skill(&mut self, name: &str) -> Result<()> {
        if self.skills.contains_key(name) {
            self.active_skill = Some(name.to_string());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Skill '{}' not found", name))
        }
    }

    pub fn deactivate_skill(&mut self) {
        self.active_skill = None;
    }

    pub fn get_active_skill(&self) -> Option<&Skill> {
        self.active_skill
            .as_ref()
            .and_then(|name| self.skills.get(name))
    }
}
