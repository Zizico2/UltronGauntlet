use const_format::formatcp;

use reqwest::{header::HeaderValue, Request};
use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

const HEADER_KEY: &str = "Content-Type";
const OLD_HEADER_VALUE: &str = "text/html";
const NEW_HEADER_VALUE: &str = formatcp!("{}; charset=windows-1252", OLD_HEADER_VALUE);

pub(crate) struct HtmlCharsetWindows1252;

#[async_trait::async_trait]
impl Middleware for HtmlCharsetWindows1252 {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        let mut response = next.run(req, extensions).await?;
        let headers = response.headers_mut();

        if headers.get(HEADER_KEY) == Some(&HeaderValue::from_static(OLD_HEADER_VALUE)) {
            headers.insert(HEADER_KEY, HeaderValue::from_static(NEW_HEADER_VALUE));
        }
        Result::Ok(response)
    }
}
