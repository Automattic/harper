#!/usr/bin/env python3
"""
Script to scrape German Wikipedia articles for testing Harper's German language support.

This script:
1. Fetches German Wikipedia articles on various topics
2. Extracts clean text content
3. Saves articles to text files in the german test_sources directory
4. Can also analyze word frequency and identify missing words
"""

import requests
import re
import os
import sys
from pathlib import Path
from collections import Counter
import argparse
from bs4 import BeautifulSoup
import html

# German Wikipedia API and URLs
WIKI_API_URL = "https://de.wikipedia.org/w/api.php"
WIKI_BASE_URL = "https://de.wikipedia.org"

# Common German stop words to filter out (optional)
STOP_WORDS = {
    'der', 'die', 'das', 'den', 'dem', 'des', 'und', 'in', 'an', 'zu', 'mit', 'von', 'für', 
    'ist', 'sind', 'war', 'waren', 'hat', 'haben', 'wurde', 'wurden', 'wird', 'werden',
    'eine', 'einer', 'einem', 'einen', 'eines', 'eine', 'ein', 'als', 'auch', 'es', 'sich',
    'nicht', 'oder', 'aber', 'dass', 'wenn', 'dann', 'weil', 'dass', 'was', 'wer', 'wie',
    'wo', 'wann', 'woher', 'warum', 'ob', 'dass', 'da', 'dort', 'hier', 'dort', 'schon',
    'noch', 'schon', 'nur', 'auch', 'sogar', 'sowohl', 'als', 'auch'
}

# Patterns to clean text
CLEAN_PATTERNS = [
    (r'\[.*?\]', ''),  # Remove citations [1], [2], etc.
    (r'\{\{.*?\}\}', ''),  # Remove template syntax {{}}
    (r'<.*?>', ''),  # Remove any remaining HTML tags
    (r'&\w+;', ''),  # Remove HTML entities
    (r'\s+', ' '),  # Collapse multiple whitespace
    (r'["""'"'"']', ''),  # Remove excessive quotes
    (r'\b[A-Za-z][A-Za-z]\.\b', ''),  # Remove single-letter abbreviations
]

def clean_text(text):
    """Clean extracted Wikipedia text."""
    text = html.unescape(text)
    for pattern, replacement in CLEAN_PATTERNS:
        text = re.sub(pattern, replacement, text)
    return text.strip()

def extract_article_text(soup):
    """Extract the main content text from a Wikipedia article."""
    # Find the main content div
    content_div = soup.find('div', {'id': 'mw-content-text'})
    if not content_div:
        return ""
    
    # Remove unwanted elements
    for element in content_div.find_all(['div', 'span', 'table', 'ul', 'ol', 'li', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6']):
        if element.name in ['div', 'span'] and 'mw-' in element.get('class', []):
            element.decompose()
    
    # Get text and clean it
    text = content_div.get_text()
    return clean_text(text)

def get_wikipedia_article(title):
    """Fetch a Wikipedia article by title."""
    params = {
        'action': 'parse',
        'page': title,
        'format': 'json',
        'prop': 'text',
        'section': 0,
        'utf8': ''
    }
    
    try:
        response = requests.get(WIKI_API_URL, params=params, timeout=30)
        response.raise_for_status()
        data = response.json()
        
        if 'parse' in data and 'text' in data['parse']:
            html_content = data['parse']['text']['*']
            soup = BeautifulSoup(html_content, 'html.parser')
            return extract_article_text(soup)
        
    except Exception as e:
        print(f"Error fetching article '{title}': {e}")
    
    return None

def get_random_articles(count=5, min_length=500):
    """Get random German Wikipedia articles."""
    articles = []
    
    # List of interesting topics for German Wikipedia
    topics = [
        'Deutschland', 'Berlin', 'München', 'Hamburg', 'Kölner Dom', 'Bundesrepublik Deutschland',
        'Deutsche Geschichte', 'Deutsche Literatur', 'Deutsche Sprache', 'Goethe', 'Schiller',
        'Martin Luther', 'Immanuel Kant', 'Albert Einstein', 'Deutsche Wissenschaft',
        'Deutsche Wirtschaft', 'Automobilindustrie', 'Siemens', 'BASF', 'Deutsche Bahn',
        'Fußball', 'Bundesliga', 'Deutsche Fußballnationalmannschaft', 'Olympische Spiele',
        'Deutsche Musik', 'Bach', 'Beethoven', 'Mozart', 'Richard Wagner',
        'Deutsche Küche', 'Brot', 'Bier', 'Wein', 'Kartoffel',
        'Deutsche Erfindungen', 'Buchdruck', 'Automobil', 'Aspirin', 'Röntgenstrahlen',
        'Umweltschutz', 'Klimawandel', 'Energiewende', 'Erneuerbare Energien',
        'Digitalisierung', 'Künstliche Intelligenz', 'Industrie 4.0',
        'Medizin', 'Gesundheitssystem', 'Bildungssystem', 'Universitäten',
        'Architektur', 'Bauhaus', 'Moderne Architektur', 'Nachhaltiges Bauen'
    ]
    
    for topic in topics[:count]:
        content = get_wikipedia_article(topic)
        if content and len(content) >= min_length:
            articles.append((topic, content))
    
    return articles

def extract_words(text):
    """Extract words from text, handling German-specific characters."""
    # German word pattern: includes umlauts and sharp s
    word_pattern = r'\b[A-ZÄÖÜa-zäöüß]+\b'
    words = re.findall(word_pattern, text.lower())
    return words

def get_missing_words(text, language='german'):
    """Identify words that Harper doesn't recognize."""
    import subprocess
    
    # Use the harper-cli to check words
    words = extract_words(text)
    missing_words = []
    
    # Test in batches to avoid too many subprocess calls
    batch_size = 20
    for i in range(0, len(words), batch_size):
        batch = words[i:i + batch_size]
        text_batch = ' '.join(batch)
        
        try:
            result = subprocess.run(
                ['just', 'language-test', language, text_batch],
                capture_output=True, text=True, timeout=60
            )
            
            # Parse output to find unrecognized words
            # This is a simplified approach - would need to parse Harper's output format
            if 'error' in result.stdout.lower() or 'unknown' in result.stdout.lower():
                # For now, just return all words that might be missing
                # In practice, you'd parse Harper's specific output format
                missing_words.extend(batch)
                
        except subprocess.TimeoutExpired:
            print(f"Timeout checking batch {i//batch_size}")
            break
        except FileNotFoundError:
            print("Harper CLI not found. Make sure to run 'just language-build' first.")
            return words  # Return all words as potentially missing
    
    return list(set(missing_words))  # Remove duplicates

def save_articles(articles, output_dir):
    """Save articles to text files in the output directory."""
    Path(output_dir).mkdir(parents=True, exist_ok=True)
    
    for title, content in articles:
        # Create a safe filename
        safe_title = re.sub(r'[^\w\s-]', '_', title).strip()
        safe_title = re.sub(r'[\s]+', '_', safe_title)
        filename = f"{safe_title}.txt"
        filepath = Path(output_dir) / filename
        
        # Save the content
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(f"# {title}\n\n")
            f.write(content)
        
        print(f"Saved: {filepath} ({len(content)} characters)")

def analyze_word_frequency(articles):
    """Analyze word frequency across all articles."""
    all_words = []
    
    for title, content in articles:
        words = extract_words(content)
        all_words.extend(words)
    
    # Filter out stop words and short words
    filtered_words = [word for word in all_words if word not in STOP_WORDS and len(word) > 2]
    
    word_counts = Counter(filtered_words)
    return word_counts

def main():
    parser = argparse.ArgumentParser(description='Scrape German Wikipedia articles for Harper testing')
    parser.add_argument('--count', type=int, default=5, help='Number of articles to scrape')
    parser.add_argument('--output', type=str, default='harper-core/src/language/german/test_sources', 
                        help='Output directory for scraped articles')
    parser.add_argument('--min-length', type=int, default=500, help='Minimum character length for articles')
    parser.add_argument('--analyze', action='store_true', help='Analyze word frequency instead of saving articles')
    parser.add_argument('--missing-words', action='store_true', help='Identify missing words in Harper dictionary')
    parser.add_argument('--topics', nargs='+', default=[], help='Specific topics to scrape')
    
    args = parser.parse_args()
    
    print("🌍 Scraping German Wikipedia articles...")
    
    if args.topics:
        # Scrape specific topics
        articles = []
        for topic in args.topics:
            content = get_wikipedia_article(topic)
            if content and len(content) >= args.min_length:
                articles.append((topic, content))
    else:
        # Scrape random articles
        articles = get_random_articles(args.count, args.min_length)
    
    if not articles:
        print("❌ No articles were scraped. Check your internet connection and Wikipedia access.")
        return
    
    print(f"✅ Scraped {len(articles)} articles")
    
    if args.analyze:
        # Analyze word frequency
        word_counts = analyze_word_frequency(articles)
        print(f"\n📊 Word Frequency Analysis:")
        print(f"Total words: {sum(word_counts.values())}")
        print(f"Unique words: {len(word_counts)}")
        print(f"\nTop 50 most frequent words:")
        for word, count in word_counts.most_common(50):
            print(f"  {word}: {count}")
    
    elif args.missing_words:
        # Identify missing words
        print("\n🔍 Identifying missing words in Harper dictionary...")
        for title, content in articles:
            print(f"\nAnalyzing: {title}")
            missing = get_missing_words(content)
            print(f"  Found {len(missing)} potentially missing words")
            if missing:
                print(f"  Sample missing words: {missing[:20]}")
    
    else:
        # Save articles
        save_articles(articles, args.output)
        print(f"\n✅ Articles saved to {args.output}")

if __name__ == '__main__':
    main()