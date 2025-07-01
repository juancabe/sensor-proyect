pub mod api;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_exists() {
        let _ = api::endpoints::post_aht10_data::PostAht10 {};
    }
}
