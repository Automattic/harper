class ParserLikeExample {
    static void main(String[] args) {
        String url = "https://example.com/path?foo=bar//baz"
        String slashy = /https?:\/\/[a-z0-9\.-]+\/.*/

        // This commment should be linted, but string and regex contents should not.
        if (url ==~ slashy) {
            println "matched"
        }
    }
}
