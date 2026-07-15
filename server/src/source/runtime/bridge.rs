use super::{SourceInstance, SourceRuntimeError};
use crate::source::{
    Chapter, FilterValue, ImageRef, ImageRequest, ImageResponse, Listing, Manga, MangaPageResult,
    Page, PageContext,
};

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

    pub fn materialize_page_image(
        &mut self,
        url: String,
        context: Option<PageContext>,
    ) -> Result<Vec<u8>, SourceRuntimeError> {
        let url_descriptor = self.store_bytes(url.as_bytes().to_vec());
        let context_descriptor = match context.as_ref() {
            Some(context) => match self.store(context) {
                Ok(descriptor) => descriptor,
                Err(error) => {
                    self.remove_descriptor(url_descriptor);
                    return Err(error);
                }
            },
            None => -1,
        };

        let request_descriptor = if self.has_export("get_image_request") {
            self.invoke2::<i32>("get_image_request", url_descriptor, context_descriptor)
        } else {
            self.create_default_image_request(&url)
        };
        self.remove_descriptor(url_descriptor);
        let request_descriptor = match request_descriptor {
            Ok(descriptor) => descriptor,
            Err(error) => {
                if context_descriptor >= 0 {
                    self.remove_descriptor(context_descriptor);
                }
                return Err(error);
            }
        };

        let result = self.download_and_process_image(request_descriptor, context_descriptor);
        self.remove_descriptor(request_descriptor);
        if context_descriptor >= 0 {
            self.remove_descriptor(context_descriptor);
        }
        result
    }

    fn download_and_process_image(
        &mut self,
        request_descriptor: i32,
        context_descriptor: i32,
    ) -> Result<Vec<u8>, SourceRuntimeError> {
        self.send_image_request(request_descriptor)?;
        let response = self.image_response_snapshot(request_descriptor)?;
        if !self.has_export("process_page_image") {
            if !(200..300).contains(&response.code) {
                return Err(SourceRuntimeError::Execution(format!(
                    "image request returned HTTP {}",
                    response.code
                )));
            }
            return Ok(response.data);
        }

        let input_image = self.image_from_request(request_descriptor)?;
        let response_descriptor = self.store(&ImageResponse {
            code: response.code,
            headers: response.headers,
            request: ImageRequest {
                url: response.request_url,
                headers: response.request_headers,
            },
            image: ImageRef(input_image),
        })?;
        let output_image = self.invoke2::<i32>(
            "process_page_image",
            response_descriptor,
            context_descriptor,
        );
        self.remove_descriptor(response_descriptor);
        let output_image = output_image?;
        let data = self.encode_image(output_image);
        self.remove_descriptor(output_image);
        if output_image != input_image {
            self.remove_descriptor(input_image);
        }
        data
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
