use propane::model;
use propane::prelude::*;
use propane::{db::Connection, ForeignKey, Many};

#[model]
#[derive(Debug, Eq, PartialEq)]
pub struct Blog {
    pub id: i64,
    pub name: String,
}
impl Blog {
    pub fn new(id: i64, name: &str) -> Self {
        Blog {
            id,
            name: name.to_string(),
        }
    }
}

#[model]
#[derive(Debug, Eq, PartialEq)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub likes: i32,
    pub tags: Many<Tag>,
    pub blog: ForeignKey<Blog>,
}
impl Post {
    pub fn new(id: i64, title: &str, body: &str, blog: &Blog) -> Self {
        Post {
            id,
            title: title.to_string(),
            body: body.to_string(),
            published: false,
            likes: 0,
            tags: Many::new(),
            blog: ForeignKey::from(blog),
        }
    }
}

#[model]
#[derive(Debug)]
pub struct Tag {
    #[pk]
    pub tag: String,
}
impl Tag {
    pub fn new(tag: &str) -> Self {
        Tag {
            tag: tag.to_string(),
        }
    }
}

/// Sets up two blogs
/// 1. "Cats"
/// 2. "Mountains"
#[allow(dead_code)] // only used by some test files
pub fn setup_blog(conn: &Connection) {
    let mut cats_blog = Blog::new(1, "Cats");
    cats_blog.save(conn).unwrap();
    let mut mountains_blog = Blog::new(2, "Mountains");
    mountains_blog.save(conn).unwrap();

    let mut tag_asia = Tag::new("asia");
    tag_asia.save(conn).unwrap();
    let mut tag_danger = Tag::new("danger");
    tag_danger.save(conn).unwrap();
    let mut tag_monkeys = Tag::new("monkeys");
    tag_monkeys.save(conn).unwrap();

    let mut post = Post::new(
        1,
        "The Tiger",
        "The tiger is a cat which would very much like to eat you.",
        &cats_blog,
    );
    post.published = true;
    post.likes = 4;
    post.tags.add(&tag_danger);
    post.tags.add(&tag_asia);
    post.save(conn).unwrap();

    let mut post = Post::new(
        2,
        "Sir Charles",
        "Sir Charles (the Very Second) is a handsome orange gentleman",
        &cats_blog,
    );
    post.published = true;
    post.likes = 20;
    post.save(conn).unwrap();

    let mut post = Post::new(
        3,
        "Mount Doom",
        "You must throw the ring into Mount Doom. Then you get to ride on a cool eagle.",
        &mountains_blog,
    );
    post.published = true;
    post.likes = 10;
    post.tags.add(&tag_danger);
    post.save(conn).unwrap();

    let mut post = Post::new(
        4,
        "Mt. Everest",
        "Everest has very little air, and lately it has very many people. This post is unfinished.",
        &mountains_blog,
    );
    post.published = false;
    post.tags.add(&tag_danger);
    post.save(conn).unwrap();
}
