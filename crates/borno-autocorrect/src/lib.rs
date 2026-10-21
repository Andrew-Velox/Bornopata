//! Autocorrect for words the Avro rules cannot spell: English words typed as they are written (`facebook`, which Avro
//! would turn into nonsense). The table is `assets/autocorrect/autocorrect.tsv`, compiled into the program;
//! [`AutocorrectTable`] is Core's `AutocorrectSource`.

mod suffix;
mod table;

#[cfg(test)]
mod tests;

pub use table::AutocorrectTable;
