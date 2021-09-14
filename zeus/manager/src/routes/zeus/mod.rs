mod dp;

pub fn mount(builder: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    let builder = self::dp::mount(builder);
    builder
}
