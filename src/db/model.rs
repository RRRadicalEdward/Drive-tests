use image::{io::Reader as ImageReader, ImageOutputFormat};
use serde::{Deserialize, Serialize};

use crate::db::schema::{tests, users};

#[derive(Queryable, Deserialize, Insertable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub name: String,
    pub second_name: String,
    pub password: String,
    pub scores: i32,
}

#[derive(Queryable, Deserialize, Insertable)]
#[table_name = "tests"]
pub struct Test {
    pub id: i32,
    pub description: String,
    pub answers: String,
    pub right_answer_id: i32,
    pub image: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserForm {
    pub name: String,
    pub second_name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct TestForm {
    pub description: String,
    pub answers: Vec<String>,
    pub right_answer_id: i32,
    pub image_path: Option<String>,
}

impl TestForm {
    pub fn into_test(self) -> anyhow::Result<Test> {
        let answers = serde_json::to_string(&self.answers)?;
        let mut image = None;

        if self.image_path.is_some() {
            let image_path = self.image_path.as_ref().unwrap();
            let image_quality = 75;

            let image_output_format = if image_path.ends_with(".jpeg") {
                ImageOutputFormat::Jpeg(image_quality)
            } else {
                ImageOutputFormat::Png
            };

            let mut buffer: Vec<u8> = Vec::new();

            ImageReader::open(image_path)?
                .decode()?
                .write_to(&mut buffer, image_output_format)?;

            image = Some(buffer);
        }

        Ok(Test {
            id: 0,
            description: self.description,
            answers,
            right_answer_id: self.right_answer_id,
            image,
        })
    }
}
