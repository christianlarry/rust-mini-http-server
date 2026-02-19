//! Template engine integration using Tera.
//!
//! Provides a thin wrapper around the Tera template engine for rendering
//! HTML templates with context data.

use std::collections::HashMap;

use crate::error::{Error, Result};

/// Template engine wrapper around Tera.
///
/// # Example
/// ```no_run
/// use mini_http::template::TemplateEngine;
///
/// let engine = TemplateEngine::new("templates/**/*.html").unwrap();
///
/// // Render with context data
/// let mut ctx = std::collections::HashMap::new();
/// ctx.insert("name".to_string(), "World".to_string());
/// let html = engine.render("index.html", &ctx).unwrap();
/// ```
pub struct TemplateEngine {
    tera: tera::Tera,
}

impl TemplateEngine {
    /// Create a new template engine from a glob pattern.
    ///
    /// # Errors
    /// Returns `Error::Template` if template files cannot be loaded or parsed.
    pub fn new(glob: &str) -> Result<Self> {
        let tera = tera::Tera::new(glob)
            .map_err(|e| Error::Template(format!("Failed to load templates: {}", e)))?;

        Ok(TemplateEngine { tera })
    }

    /// Render a template with string context data.
    pub fn render(&self, template: &str, data: &HashMap<String, String>) -> Result<String> {
        let mut context = tera::Context::new();
        for (key, value) in data {
            context.insert(key, value);
        }

        self.tera
            .render(template, &context)
            .map_err(|e| Error::Template(format!("Render error: {}", e)))
    }

    /// Render a template with a Tera context (for complex data types).
    pub fn render_with_context(
        &self,
        template: &str,
        context: &tera::Context,
    ) -> Result<String> {
        self.tera
            .render(template, context)
            .map_err(|e| Error::Template(format!("Render error: {}", e)))
    }

    /// Render a template with serializable data.
    pub fn render_with<T: serde::Serialize>(
        &self,
        template: &str,
        data: &T,
    ) -> Result<String> {
        let context = tera::Context::from_serialize(data)
            .map_err(|e| Error::Template(format!("Context error: {}", e)))?;

        self.tera
            .render(template, &context)
            .map_err(|e| Error::Template(format!("Render error: {}", e)))
    }

    /// Register a custom Tera template from a string.
    pub fn add_template(&mut self, name: &str, content: &str) -> Result<()> {
        self.tera
            .add_raw_template(name, content)
            .map_err(|e| Error::Template(format!("Failed to add template: {}", e)))
    }

    /// Get a reference to the underlying Tera engine (for advanced configuration).
    pub fn tera(&self) -> &tera::Tera {
        &self.tera
    }

    /// Get a mutable reference to the underlying Tera engine.
    pub fn tera_mut(&mut self) -> &mut tera::Tera {
        &mut self.tera
    }
}
