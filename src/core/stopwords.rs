/// 共享停用词表 (中英文)
/// 包含中文虚词、英文介词/代词/助动词/连词
pub const STOPWORDS: &[&str] = &[
    // 中文虚词
    "的", "是", "了", "在", "我", "你", "他", "她", "它", "们", "这", "那", "都", "和", "并", "且",
    "也", "就", "着", "吧", "吗", "呢", "啊", "呀", "呜", "哎", "哼", "呸", "喽",
    // 英文介词
    "a", "an", "the", "about", "above", "across", "after", "against", "along", "among", "around", "at", 
    "before", "behind", "below", "beneath", "beside", "between", "beyond", "but", "by", "despite", "down", 
    "during", "except", "for", "from", "in", "inside", "into", "like", "near", "of", "off", "on", "onto", 
    "out", "outside", "over", "past", "since", "through", "throughout", "till", "to", "toward", "under", 
    "underneath", "until", "up", "upon", "with", "within", "without",
    // 英文代词
    "i", "me", "my", "mine", "we", "us", "our", "ours", "you", "your", "yours", "he", "him", "his", 
    "she", "her", "hers", "it", "its", "they", "them", "their", "theirs", "this", "that", "these", "those", 
    "who", "whom", "whose", "which", "what", "each", "every", "either", "neither", "some", "any", "no", 
    "none", "both", "few", "many", "other", "another",
    // 英文助动词
    "am", "is", "are", "was", "were", "be", "being", "been", "have", "has", "had", "do", "does", "did", 
    "shall", "will", "should", "would", "may", "might", "must", "can", "could",
    // 英文连词及其他
    "and", "or", "so", "nor", "yet", "although", "because", "unless", "while", "where", "when", "how", "whether"
];

/// 判断是否为停用词
#[inline]
pub fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(&word)
}
