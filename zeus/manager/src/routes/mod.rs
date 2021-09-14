pub mod zeus;

pub fn mount(builder: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    let builder = self::zeus::mount(builder);
    builder
}
