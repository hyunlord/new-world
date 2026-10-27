//! Additive story diagnostic command; original story-yield instrument is unchanged.
mod capture;
mod cli;
pub use cli::run;

#[cfg(test)]
mod tests;
