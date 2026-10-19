//! The scaffold source answers through the trait object, as Core will call it.

use borno_transliteration::{UpodeshWordSource, WordSource};

#[test]
fn upodesh_suggests_ami_through_the_trait() {
    let source: Box<dyn WordSource> = Box::new(UpodeshWordSource::new());
    let words = source.suggest("ami");
    assert!(words.iter().any(|word| word == "আমি"), "got {words:?}");
}
