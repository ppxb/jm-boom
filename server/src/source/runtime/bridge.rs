use super::{SourceInstance, SourceRuntimeError};
use crate::source::{Chapter, FilterValue, Listing, Manga, MangaPageResult, Page};

impl SourceInstance {
    pub fn search(
        &mut self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult, SourceRuntimeError> {
        let query = self.store_bytes(query.unwrap_or_default().into_bytes());
        let filters = match self.store(&filters) {
            Ok(descriptor) => descriptor,
            Err(error) => {
                self.remove_descriptor(query);
                return Err(error);
            }
        };
        let result = self.invoke3("get_search_manga_list", query, page, filters);
        self.remove_descriptor(query);
        self.remove_descriptor(filters);
        result
    }

    pub fn update_manga(
        &mut self,
        manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga, SourceRuntimeError> {
        let manga = self.store(&manga)?;
        let result = self.invoke3(
            "get_manga_update",
            manga,
            i32::from(needs_details),
            i32::from(needs_chapters),
        );
        self.remove_descriptor(manga);
        result
    }

    pub fn get_pages(
        &mut self,
        manga: Manga,
        chapter: Chapter,
    ) -> Result<Vec<Page>, SourceRuntimeError> {
        let manga = self.store(&manga)?;
        let chapter = match self.store(&chapter) {
            Ok(descriptor) => descriptor,
            Err(error) => {
                self.remove_descriptor(manga);
                return Err(error);
            }
        };
        let result = self.invoke2("get_page_list", manga, chapter);
        self.remove_descriptor(manga);
        self.remove_descriptor(chapter);
        result
    }

    pub fn get_listing(
        &mut self,
        listing: Listing,
        page: i32,
    ) -> Result<MangaPageResult, SourceRuntimeError> {
        let listing = self.store(&listing)?;
        let result = self.invoke2("get_manga_list", listing, page);
        self.remove_descriptor(listing);
        result
    }
}
