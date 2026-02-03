//! 技术栈检测器
//!
//! 从项目文档（CLAUDE.md、README.md）中自动检测技术栈

use std::path::Path;
use std::fs;
use regex::Regex;
use anyhow::{Result, Context};

/// 技术栈检测器
///
/// 使用关键词字典 + 正则表达式从项目文档中检测技术栈
pub struct TechStackDetector {
    /// 技术栈关键词字典（关键词 → 技术栈名称）
    keywords: Vec<(&'static str, &'static str)>,
    /// 预编译正则表达式
    regex: Regex,
}

impl TechStackDetector {
    /// 创建新的技术栈检测器
    pub fn new() -> Result<Self> {
        // 构建技术栈关键词字典（热门技术栈）
        let keywords = vec![
            // 编程语言
            (r"\bRust\b", "Rust"),
            (r"\bGo\b", "Go"),
            (r"\bGolang\b", "Go"),
            (r"\bPython\b", "Python"),
            (r"\bJava\b", "Java"),
            (r"\bKotlin\b", "Kotlin"),
            (r"\bSwift\b", "Swift"),
            (r"\bC\+\+\b", "C++"),
            (r"\bC\s*#", "C#"),
            (r"\bJavaScript\b", "JavaScript"),
            (r"\bTypeScript\b", "TypeScript"),
            (r"\bRuby\b", "Ruby"),
            (r"\bPHP\b", "PHP"),
            (r"\bScala\b", "Scala"),
            (r"\bR\b", "R"),

            // 前端框架
            (r"\bReact\b", "React"),
            (r"\bVue\b", "Vue"),
            (r"\bAngular\b", "Angular"),
            (r"\bSvelte\b", "Svelte"),
            (r"\bSolid\b", "Solid"),
            (r"\bQwik\b", "Qwik"),
            (r"\bNext\.?js\b", "Next.js"),
            (r"\bNuxt\b", "Nuxt"),
            (r"\bRemix\b", "Remix"),
            (r"\bAstro\b", "Astro"),

            // 后端框架
            (r"\bSpring\s*Boot\b", "Spring Boot"),
            (r"\bExpress\b", "Express"),
            (r"\bFastAPI\b", "FastAPI"),
            (r"\bDjango\b", "Django"),
            (r"\bFlask\b", "Flask"),
            (r"\bRails\b", "Ruby on Rails"),
            (r"\bLaravel\b", "Laravel"),
            (r"\bActix\b", "Actix"),
            (r"\bRocket\b", "Rocket"),
            (r"\bAxum\b", "Axum"),

            // 桌面/移动框架
            (r"\bTauri\b", "Tauri"),
            (r"\bElectron\b", "Electron"),
            (r"\bFlutter\b", "Flutter"),
            (r"\bReact\s*Native\b", "React Native"),
            (r"\bSwiftUI\b", "SwiftUI"),

            // 构建工具
            (r"\bVite\b", "Vite"),
            (r"\bWebpack\b", "Webpack"),
            (r"\bRollup\b", "Rollup"),
            (r"\bParcel\b", "Parcel"),
            (r"\bEsbuild\b", "Esbuild"),
            (r"\bTurbopack\b", "Turbopack"),
            (r"\bCargo\b", "Cargo"),
            (r"\bGradle\b", "Gradle"),
            (r"\bMaven\b", "Maven"),
            (r"\bnpm\b", "npm"),
            (r"\byarn\b", "Yarn"),
            (r"\bpnpm\b", "pnpm"),

            // 数据库
            (r"\bPostgreSQL\b", "PostgreSQL"),
            (r"\bMySQL\b", "MySQL"),
            (r"\bSQLite\b", "SQLite"),
            (r"\bMongoDB\b", "MongoDB"),
            (r"\bRedis\b", "Redis"),
            (r"\bDynamoDB\b", "DynamoDB"),
            (r"\bCassandra\b", "Cassandra"),

            // ORM/数据库工具
            (r"\bPrisma\b", "Prisma"),
            (r"\bDrizzle\b", "Drizzle"),
            (r"\bSQLx\b", "SQLx"),
            (r"\bDiesel\b", "Diesel"),
            (r"\bSequelize\b", "Sequelize"),
            (r"\bTypeORM\b", "TypeORM"),
            (r"\bHibernate\b", "Hibernate"),
            (r"\b SQLAlchemy\b", "SQLAlchemy"),

            // 异步运行时
            (r"\bTokio\b", "Tokio"),
            (r"\basyncio\b", "asyncio"),
            (r"\bNode\b", "Node.js"),

            // 序列化/数据格式
            (r"\bSerde\b", "Serde"),
            (r"\bJSON\b", "JSON"),
            (r"\bYAML\b", "YAML"),
            (r"\bTOML\b", "TOML"),

            // 测试框架
            (r"\bJest\b", "Jest"),
            (r"\bVitest\b", "Vitest"),
            (r"\bPytest\b", "Pytest"),
            (r"\bJUnit\b", "JUnit"),

            // HTTP 客户端
            (r"\bAxum\b", "Axum"),
            (r"\bReqwest\b", "Reqwest"),
            (r"\bFetch\b", "Fetch API"),
            (r"\bAxios\b", "Axios"),
            (r"\bCurl\b", "cURL"),

            // UI 组件库
            (r"\bTailwind\b", "Tailwind CSS"),
            (r"\bBootstrap\b", "Bootstrap"),
            (r"\bMaterial\s*UI\b", "Material-UI"),
            (r"\bAnt\s*Design\b", "Ant Design"),
            (r"\bChakra\b", "Chakra UI"),

            // 状态管理
            (r"\bRedux\b", "Redux"),
            (r"\bZustand\b", "Zustand"),
            (r"\bMobX\b", "MobX"),
            (r"\bRecoil\b", "Recoil"),
            (r"\bPinia\b", "Pinia"),

            // API/GraphQL
            (r"\bGraphQL\b", "GraphQL"),
            (r"\bREST\b", "REST API"),
            (r"\bgRPC\b", "gRPC"),
            (r"\bOpenAPI\b", "OpenAPI"),
            (r"\bSwagger\b", "Swagger"),

            // 容器/部署
            (r"\bDocker\b", "Docker"),
            (r"\bKubernetes\b", "Kubernetes"),
            (r"\bK8s\b", "Kubernetes"),
            (r"\bCI/?CD\b", "CI/CD"),

            // 云服务
            (r"\bAWS\b", "AWS"),
            (r"\bAzure\b", "Azure"),
            (r"\bGCP\b", "Google Cloud"),
            (r"\bVercel\b", "Vercel"),
            (r"\bNetlify\b", "Netlify"),
            (r"\bRailway\b", "Railway"),

            // 其他工具
            (r"\bGit\b", "Git"),
            (r"\bGitHub\b", "GitHub"),
            (r"\bGitLab\b", "GitLab"),
            (r"\bVS\s*Code\b", "VS Code"),
            (r"\bESLint\b", "ESLint"),
            (r"\bPrettier\b", "Prettier"),
        ];

        // 构建正则表达式（使用非捕获组提高性能）
        let pattern = keywords.iter()
            .map(|(regex, _)| format!("(?:{})", regex))
            .collect::<Vec<_>>()
            .join("|");

        let regex = Regex::new(&pattern)
            .context("构建技术栈检测正则表达式失败")?;

        Ok(Self { keywords, regex })
    }

    /// 从项目文档中检测技术栈
    ///
    /// 检测顺序：CLAUDE.md → README.md
    ///
    /// # Arguments
    ///
    /// * `project_path` - 项目路径
    ///
    /// # Returns
    ///
    /// 返回检测到的技术栈列表（已去重）
    pub fn detect_from_project(&self, project_path: &str) -> Vec<String> {
        let path = Path::new(project_path);
        let mut tech_stack = Vec::new();

        // 优先检测 CLAUDE.md
        let claude_md = path.join("CLAUDE.md");
        if claude_md.exists() {
            if let Ok(content) = fs::read_to_string(&claude_md) {
                tech_stack.extend(self.detect_from_text(&content));
            }
        }

        // 检测 README.md
        let readme_md = path.join("README.md");
        if readme_md.exists() {
            if let Ok(content) = fs::read_to_string(&readme_md) {
                tech_stack.extend(self.detect_from_text(&content));
            }
        }

        // 去重（保持顺序）
        tech_stack.sort();
        tech_stack.dedup();
        tech_stack
    }

    /// 从文本中检测技术栈
    ///
    /// 使用正则表达式快速匹配关键词
    fn detect_from_text(&self, text: &str) -> Vec<String> {
        let mut tech_stack = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // 使用正则表达式查找所有匹配
        for caps in self.regex.captures_iter(text) {
            if let Some(full_match) = caps.get(0) {
                let matched_text = full_match.as_str();

                // 查找匹配的技术栈名称
                for (regex, tech_name) in &self.keywords {
                    if let Ok(re) = Regex::new(regex) {
                        if re.is_match(matched_text) {
                            if seen.insert(tech_name.to_string()) {
                                tech_stack.push(tech_name.to_string());
                            }
                            break;
                        }
                    }
                }
            }
        }

        tech_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust_project() {
        let detector = TechStackDetector::new().unwrap();
        let text = r#"
# CLAUDE.md

This project is built with Rust and Tauri.

## Tech Stack

- Rust
- Tauri 2.0
- SQLite
"#;
        let result = detector.detect_from_text(text);
        assert!(result.contains(&"Rust".to_string()));
        assert!(result.contains(&"Tauri".to_string()));
        assert!(result.contains(&"SQLite".to_string()));
    }

    #[test]
    fn test_detect_nodejs_project() {
        let detector = TechStackDetector::new().unwrap();
        let text = r#"
# README.md

A React + Vite project with TypeScript.

## Dependencies

- React 18
- TypeScript
- Vite 5
- Tailwind CSS
"#;
        let result = detector.detect_from_text(text);
        assert!(result.contains(&"React".to_string()));
        assert!(result.contains(&"TypeScript".to_string()));
        assert!(result.contains(&"Vite".to_string()));
        assert!(result.contains(&"Tailwind CSS".to_string()));
    }

    #[test]
    fn test_detect_python_project() {
        let detector = TechStackDetector::new().unwrap();
        let text = r#"
# CLAUDE.md

Python project using FastAPI and PostgreSQL.

## Requirements

- fastapi
- sqlalchemy
- pytest
"#;
        let result = detector.detect_from_text(text);
        assert!(result.contains(&"Python".to_string()));
        assert!(result.contains(&"FastAPI".to_string()));
        assert!(result.contains(&"PostgreSQL".to_string()));
        assert!(result.contains(&"SQLAlchemy".to_string()));
        assert!(result.contains(&"Pytest".to_string()));
    }

    #[test]
    fn test_deduplication() {
        let detector = TechStackDetector::new().unwrap();
        let text = "Rust Rust React React";
        let result = detector.detect_from_text(text);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_detect_from_project_files() {
        let detector = TechStackDetector::new().unwrap();
        let temp_dir = std::env::temp_dir();
        let project_path = temp_dir.join("test_project");
        fs::create_dir_all(&project_path).unwrap();

        // 创建 CLAUDE.md
        fs::write(
            project_path.join("CLAUDE.md"),
            "Rust Tauri SQLite"
        ).unwrap();

        // 创建 README.md
        fs::write(
            project_path.join("README.md"),
            "React TypeScript Vite"
        ).unwrap();

        let result = detector.detect_from_project(project_path.to_str().unwrap());
        assert!(result.contains(&"Rust".to_string()));
        assert!(result.contains(&"Tauri".to_string()));
        assert!(result.contains(&"React".to_string()));
        assert!(result.contains(&"TypeScript".to_string()));

        // 清理
        fs::remove_file(project_path.join("CLAUDE.md")).unwrap();
        fs::remove_file(project_path.join("README.md")).unwrap();
        fs::remove_dir(&project_path).unwrap();
    }

    #[test]
    fn test_empty_text() {
        let detector = TechStackDetector::new().unwrap();
        let result = detector.detect_from_text("");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_new_success() {
        let detector = TechStackDetector::new();
        assert!(detector.is_ok());
    }
}
