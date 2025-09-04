#[cfg(test)]
mod tests {
    use super::super::cache::cache;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = cache();
        cache.set("test_key", "test_value").await;
        let ret = cache.get("test_key").await;
        assert_eq!(ret, Some("test_value".to_string()));
        let ret = cache.delete("test_key").await;
        assert_eq!(ret, Some(()));

        let ret = cache.get("test_key").await;
        assert_eq!(ret, None);
        let ret = cache.delete("test_key").await;
        assert_eq!(ret, None);
    }
}
