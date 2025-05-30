use std::sync::Arc;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::ai_utils::Tool;

use super::{
    chapter::{Chapter, ChapterNumber},
    library::Library,
};

pub struct GetChapterTool {
    book_id: i64,
    library: Arc<Library>,
}

impl GetChapterTool {
    pub fn new(book_id: i64, library: Arc<Library>) -> Self {
        Self { book_id, library }
    }
}

impl Tool for GetChapterTool {
    type Args = ChapterNumber;
    type Output = Chapter;
    fn name(&self) -> String {
        "GetChapterContent".to_string()
    }
    fn description(&self) -> Option<String> {
        Some(
            "Query the content of a chapter from the book. \
            Before starting to teach a new chapter, use this tool to get the content of this chapter"
                .to_string(),
        )
    }
    async fn call(&self, args: Self::Args) -> anyhow::Result<Self::Output> {
        let book = self.library.get_book(self.book_id).await?;
        let chapter = book
            .chapters
            .get(&args)
            .ok_or(anyhow::anyhow!("Chapter not found: {:?}", args))?;
        Ok(chapter.clone())
    }
}
#[tokio::test]
async fn t() {
    let db = sqlx::SqlitePool::connect(":memory:").await.unwrap();
    let library = Arc::new(Library {
        books: moka::future::Cache::new(1000),
        bookbase: std::path::PathBuf::new(),
        database: db,
    });
    let tool = BookJumpTool::new(1, library);
    println!("{:#?}", tool.definition());
}
/// Specifies a location in the book by chapter number and optional section title
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BookLocation {
    /// The chapter number to navigate to
    pub chapter_number: ChapterNumber,
    /// Optional section title within the chapter
    pub sector_title: Option<String>,
}

pub struct BookJumpTool {
    book_id: i64,
    library: Arc<Library>,
}

impl BookJumpTool {
    pub fn new(book_id: i64, library: Arc<Library>) -> Self {
        Self { book_id, library }
    }
}

impl Tool for BookJumpTool {
    type Args = BookLocation;
    type Output = String;
    fn name(&self) -> String {
        "BookJump".to_string()
    }
    fn description(&self) -> Option<String> {
        Some(
            "Use this tool to navigate to a specific chapter or section in the book \
             when you need the student to read particular content. It helps direct the \
             student's attention to the relevant material."
                .to_string(),
        )
    }
    async fn call(&self, args: Self::Args) -> anyhow::Result<Self::Output> {
        let book = self.library.get_book(self.book_id).await?;
        let chapter = book
            .chapters
            .get(&args.chapter_number)
            .ok_or(anyhow::anyhow!(
                "Chapter not found: {:?}",
                args.chapter_number
            ))?;
        let sector_title = args
            .sector_title
            .map(|s| "#".to_string() + &s)
            .unwrap_or_default();
        Ok(format!(
            "Jumped to {} {}{}",
            args.chapter_number, chapter.name, sector_title
        ))
    }
}
