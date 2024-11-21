
use std::collections::HashSet;
use comrak::markdown_to_html;
use maud::{html, Markup, DOCTYPE};

use super::{Post, PostPreview};

fn navbar(_posts: &[PostPreview], tag_set: &HashSet<String>) -> Markup {
    html! {
        nav class="navbar" {
            div class="container" {
                // Left: Logo or brand name
                a href="/" class="navbar-brand" { "My Blog" }

                // Center: Navigation links
                ul class="navbar-nav" {
                    li class="nav-item" { a href="/" { "Home" } }
                    li class="nav-item" { a href="/blog" { "Blog" } }
                    li class="nav-item" { a href="/about" { "About" } }
                }

                // Right: Search bar
				form class="navbar-search" id="search-form" {
					input class="form-control me-2" type="search" placeholder="Search" aria-label="Search" id="search-input";
				}
            }

            // Tag filter section (Initially hidden)
            div class="tag-filter" id="tag-filter" style="display: none;" {
                p { "Filter by Tag:" }
                form id="tag-form" {
                    // Checkboxes for each tag
                    @for tag in tag_set {
                        label {
                            input type="checkbox" name="tags" value={(tag.clone())} {
                            }
                            (tag)
                        }
                    }
                    button type="submit" { "Filter" }
                }
            }
        }

        // JavaScript to show the filter when search is active
        script {
            r#"
                document.getElementById('search-input').addEventListener('focus', function() {
                    document.getElementById('tag-filter').style.display = 'block';
                });

                document.getElementById('search-input').addEventListener('blur', function() {
                    // Hide the filter when search bar loses focus (optional)
                    setTimeout(function() {
                        if (!document.getElementById('search-input').value) {
                            document.getElementById('tag-filter').style.display = 'none';
                        }
                    }, 200);
                });
            "#
        }
    }
}


fn header(current_post: &Post, posts: &[PostPreview], tag_set: &HashSet<String>) -> Markup {
    html! {
        // Metadata
        head {
            meta charset="utf-8";
            title { (current_post.parsed_post_data.data.title) }
            meta name="author" content=(current_post.parsed_post_data.data.author );
            meta name="description" content=(current_post.parsed_post_data.data.description);
            link rel="stylesheet" type="text/css" href="assets/styles.css";
        }
        header {
            // navbar
            (navbar(posts, tag_set))
        }
    }
}

fn body(blog_post: String) -> Markup {
    html! {
        body {
            (maud::PreEscaped(blog_post))
        }
    }
}

fn footer(publishing_date: &str) -> Markup {
	html! {
		footer {
			p {
				"Published on: " (publishing_date)
			}
			a href="#top" {
				"Back to top"
			}
		}
	}
}

pub fn generate_blog_post(current_post: &Post, previews: &[PostPreview], tag_set: &HashSet<String>) -> Markup {
    // Markdown options and conversion to HTML
    let options = comrak::ComrakOptions::default();
    let content = markdown_to_html(&current_post.parsed_post_data.content, &options);
    let publishing_date = &current_post.parsed_post_data.data.date;
    
    // Generate the blog post page
    html! {
        (DOCTYPE)
        html {
            (header(current_post, previews, tag_set))
            (body(content))
            (footer(publishing_date))
        }
    }
}

