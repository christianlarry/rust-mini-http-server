//! Cookie building utilities for HTTP responses.
//!
//! Provides [`CookieBuilder`] for constructing `Set-Cookie` header values
//! with all standard attributes (Path, Domain, Max-Age, Secure, HttpOnly, SameSite).

/// SameSite cookie attribute values.
#[derive(Debug, Clone, PartialEq)]
pub enum SameSite {
    /// Cookie is only sent in first-party context.
    Strict,
    /// Cookie is sent with top-level navigations and GET requests from third-party.
    Lax,
    /// Cookie is sent in all contexts (requires Secure).
    None,
}

/// Builder for `Set-Cookie` header values.
///
/// # Example
/// ```
/// use mini_http::cookie::{CookieBuilder, SameSite};
///
/// let cookie = CookieBuilder::new("session_id", "abc123")
///     .path("/")
///     .http_only()
///     .secure()
///     .same_site(SameSite::Lax)
///     .max_age(3600)
///     .build();
///
/// assert!(cookie.contains("session_id=abc123"));
/// assert!(cookie.contains("HttpOnly"));
/// ```
#[derive(Debug, Clone)]
pub struct CookieBuilder {
    name: String,
    value: String,
    path: Option<String>,
    domain: Option<String>,
    max_age: Option<i64>,
    expires: Option<String>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

impl CookieBuilder {
    /// Create a new cookie with the given name and value.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            path: None,
            domain: None,
            max_age: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// Set the Path attribute.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the Domain attribute.
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set the Max-Age attribute in seconds.
    pub fn max_age(mut self, seconds: i64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Set the Expires attribute as an HTTP-date string.
    pub fn expires(mut self, date: impl Into<String>) -> Self {
        self.expires = Some(date.into());
        self
    }

    /// Set the Secure flag (cookie only sent over HTTPS).
    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    /// Set the HttpOnly flag (cookie inaccessible to JavaScript).
    pub fn http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    /// Set the SameSite attribute.
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Build the `Set-Cookie` header string.
    pub fn build(&self) -> String {
        let mut cookie = format!("{}={}", self.name, self.value);

        if let Some(ref path) = self.path {
            cookie.push_str(&format!("; Path={}", path));
        }
        if let Some(ref domain) = self.domain {
            cookie.push_str(&format!("; Domain={}", domain));
        }
        if let Some(max_age) = self.max_age {
            cookie.push_str(&format!("; Max-Age={}", max_age));
        }
        if let Some(ref expires) = self.expires {
            cookie.push_str(&format!("; Expires={}", expires));
        }
        if self.secure {
            cookie.push_str("; Secure");
        }
        if self.http_only {
            cookie.push_str("; HttpOnly");
        }
        if let Some(ref same_site) = self.same_site {
            match same_site {
                SameSite::Strict => cookie.push_str("; SameSite=Strict"),
                SameSite::Lax => cookie.push_str("; SameSite=Lax"),
                SameSite::None => cookie.push_str("; SameSite=None"),
            }
        }

        cookie
    }

    /// Create a removal cookie (Max-Age=0).
    pub fn removal(name: impl Into<String>) -> Self {
        Self::new(name, "")
            .max_age(0)
            .path("/")
    }
}
