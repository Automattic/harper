#!/usr/bin/env python3
"""
Simple script to scrape German Wikipedia articles using wget/curl and basic parsing.
This version uses standard libraries only and is more reliable.

This script:
1. Uses urllib to fetch Wikipedia pages (no external dependencies)
2. Extracts clean text content using regex
3. Saves articles to text files for Harper testing
4. Can analyze coverage against Harper's dictionary
"""

import urllib.request
import urllib.error
import re
import os
import sys
import json
from pathlib import Path
from collections import Counter
import argparse
import subprocess
import time

# Wikipedia URL
WIKI_BASE_URL = "https://de.wikipedia.org/wiki/"

# Patterns for cleaning
CITATION_PATTERN = re.compile(r'\[[^\]]*\]')
HTML_TAG_PATTERN = re.compile(r'<[^>]*>')
HTML_ENTITY_PATTERN = re.compile(r'&[^;]*;')
TEMPLATE_PATTERN = re.compile(r'\{[^{}]*\}')
WHITESPACE_PATTERN = re.compile(r'\s+')

# Common German stop words
STOP_WORDS = {
    'der', 'die', 'das', 'den', 'dem', 'des', 'und', 'in', 'an', 'zu', 'mit', 'von', 'für', 
    'ist', 'sind', 'war', 'waren', 'hat', 'haben', 'wurde', 'wurden', 'wird', 'werden',
    'eine', 'einer', 'einem', 'einen', 'eines', 'ein', 'als', 'auch', 'es', 'sich',
    'nicht', 'oder', 'aber', 'dass', 'wenn', 'dann', 'weil', 'was', 'wer', 'wie',
    'wo', 'wann', 'woher', 'warum', 'ob', 'da', 'dort', 'hier', 'schon', 'noch',
    'nur', 'sogar', 'sowohl', 'nicht', 'sehr', 'mehr', 'viel', 'viele', 'einer'
}

def clean_text(text):
    """Clean Wikipedia text content."""
    # Remove citations [1], [2], etc.
    text = CITATION_PATTERN.sub('', text)
    # Remove templates {{...}}
    text = TEMPLATE_PATTERN.sub('', text)
    # Remove HTML tags
    text = HTML_TAG_PATTERN.sub('', text)
    # Remove HTML entities
    text = HTML_ENTITY_PATTERN.sub('', text)
    # Decode HTML entities that might remain
    text = text.replace('&nbsp;', ' ')
    # Collapse whitespace
    text = WHITESPACE_PATTERN.sub(' ', text)
    # Remove excessive punctuation
    text = re.sub(r'["""'"'"']+', '', text)
    
    return text.strip()

def extract_main_content(html):
    """Extract main content from Wikipedia HTML."""
    # Look for the content div
    content_match = re.search(r'<div id="mw-content-text"[^>]*>(.*?)</div>', html, re.DOTALL)
    if content_match:
        content = content_match.group(1)
        return clean_text(content)
    
    # Fallback: extract all text between <p> tags
    paragraphs = re.findall(r'<p[^>]*>(.*?)</p>', html, re.DOTALL)
    if paragraphs:
        return clean_text(' '.join(paragraphs))
    
    # Last resort: clean the whole HTML
    return clean_text(html)

def fetch_wikipedia_article(title):
    """Fetch a Wikipedia article."""
    url = WIKI_BASE_URL + title.replace(' ', '_')
    
    try:
        # Set a user agent to avoid blocking
        headers = {'User-Agent': 'Mozilla/5.0 (compatible; Harper German Language Support/1.0)'}
        req = urllib.request.Request(url, headers=headers)
        
        with urllib.request.urlopen(req, timeout=30) as response:
            html = response.read().decode('utf-8')
            return extract_main_content(html)
            
    except urllib.error.URLError as e:
        print(f"Error fetching '{title}': {e}")
    except Exception as e:
        print(f"Error processing '{title}': {e}")
    
    return None

def extract_words(text):
    """Extract German words from text."""
    # German word pattern: includes umlauts and sharp s
    word_pattern = r'[A-ZÄÖÜa-zäöüß]+'
    words = re.findall(word_pattern, text.lower())
    return words

def get_harper_coverage(text, language='german'):
    """Test text with Harper and return coverage information."""
    words = extract_words(text)
    total_words = len(words)
    
    if total_words == 0:
        return {'total': 0, 'recognized': 0, 'missing': 0, 'coverage': 0.0}
    
    # Use Harper's language test to check the text
    try:
        result = subprocess.run(
            ['just', 'language-test', language, text],
            capture_output=True, text=True, timeout=120,
            cwd='/home/konrad/gallery/harper'
        )
        
        # Check if the test was successful (no errors mentioned)
        # Harper's output format needs to be parsed properly
        output = result.stdout + result.stderr
        
        # Simple heuristic: if we see error messages, count as missing
        # This is a placeholder - the actual parsing would depend on Harper's output format
        if 'error' in output.lower() or 'unknown' in output.lower():
            # For now, we'll use a different approach
            # Test individual words
            recognized = 0
            for word in words:
                try:
                    word_result = subprocess.run(
                        ['just', 'language-meta', language, word],
                        capture_output=True, text=True, timeout=30,
                        cwd='/home/konrad/gallery/harper'
                    )
                    if 'Found' in word_result.stdout or 'metadata' in word_result.stdout.lower():
                        recognized += 1
                except:
                    pass  # Word not recognized
            
            missing = total_words - recognized
            coverage = (recognized / total_words * 100) if total_words > 0 else 0.0
            
            return {
                'total': total_words,
                'recognized': recognized,
                'missing': missing,
                'coverage': coverage
            }
        else:
            # Assume all words are recognized
            return {
                'total': total_words,
                'recognized': total_words,
                'missing': 0,
                'coverage': 100.0
            }
            
    except subprocess.TimeoutExpired:
        print("Harper test timed out")
        return {'total': total_words, 'recognized': 0, 'missing': total_words, 'coverage': 0.0}
    except FileNotFoundError:
        print("Harper CLI not found. Run 'just language-build' first.")
        return {'total': total_words, 'recognized': 0, 'missing': total_words, 'coverage': 0.0}
    except Exception as e:
        print(f"Error testing with Harper: {e}")
        return {'total': total_words, 'recognized': 0, 'missing': total_words, 'coverage': 0.0}

# List of interesting German Wikipedia topics
topics_list = [
    'Deutschland',
    'Berlin', 
    'München',
    'Hamburg',
    'Kölner_Dom',
    'Bundesrepublik_Deutschland',
    'Deutsche_Geschichte',
    'Deutsche_Literatur',
    'Deutsche_Sprache',
    'Johann_Wolfgang_von_Goethe',
    'Friedrich_von_Schiller',
    'Martin_Luther',
    'Immanuel_Kant',
    'Albert_Einstein',
    'Deutsche_Wissenschaft',
    'Deutsche_Wirtschaft',
    'Automobilindustrie',
    'Siemens',
    'BASF',
    'Deutsche_Bahn',
    'Fußball',
    'Bundesliga',
    'Deutsche_Fußballnationalmannschaft',
    'Olympische_Spiele',
    'Deutsche_Musik',
    'Johann_Sebastian_Bach',
    'Ludwig_van_Beethoven',
    'Wolfgang_Amadeus_Mozart',
    'Richard_Wagner',
    'Deutsche_Küche',
    'Brot',
    'Bier',
    'Wein',
    'Kartoffel',
    'Deutsche_Erfindungen',
    'Buchdruck',
    'Automobil',
    'Aspirin',
    'Röntgenstrahlen',
    'Umweltschutz',
    'Klimawandel',
    'Energiewende',
    'Erneuerbare_Energien',
    'Digitalisierung',
    'Künstliche_Intelligenz',
    'Industrie_4.0',
    'Medizin',
    'Gesundheitssystem',
    'Bildungssystem',
    'Universität',
    'Architektur',
    'Bauhaus',
    'Moderne_Architektur',
    'Nachhaltiges_Bauen'
]

def save_articles(articles, output_dir):
    """Save articles to text files."""
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
        
        print(f"✅ Saved: {filepath} ({len(content)} characters)")

def analyze_articles(articles):
    """Analyze articles for word frequency and coverage."""
    all_words = []
    total_coverage = 0.0
    article_count = len(articles)
    
    print(f"\n📊 Analyzing {article_count} articles...")
    
    for title, content in articles:
        words = extract_words(content)
        filtered_words = [word for word in words if word not in STOP_WORDS and len(word) > 2]
        all_words.extend(filtered_words)
        
        # Test coverage for this article
        coverage = get_harper_coverage(content)
        total_coverage += coverage['coverage']
        
        print(f"  {title}: {len(filtered_words)} words, {coverage['coverage']:.1f}% coverage")
    
    # Word frequency analysis
    word_counts = Counter(all_words)
    unique_words = len(word_counts)
    total_words = sum(word_counts.values())
    
    print(f"\n📈 Overall Statistics:")
    print(f"  Total unique words: {unique_words}")
    print(f"  Total word occurrences: {total_words}")
    print(f"  Average coverage: {total_coverage / article_count:.1f}%")
    
    print(f"\n🔝 Top 30 most frequent content words:")
    for word, count in word_counts.most_common(30):
        print(f"  {word}: {count}")
    
    return word_counts

def main():
    parser = argparse.ArgumentParser(description='Scrape German Wikipedia articles for Harper testing')
    parser.add_argument('--topics', nargs='+', default=[], 
                        help='Specific Wikipedia article titles to scrape (use underscores for spaces)')
    parser.add_argument('--count', type=int, default=10, 
                        help='Number of random articles to scrape (if no topics specified)')
    parser.add_argument('--output', type=str, 
                        default='/home/konrad/gallery/harper/harper-core/src/language/german/test_sources',
                        help='Output directory for scraped articles')
    parser.add_argument('--analyze', action='store_true', 
                        help='Analyze word frequency and Harper coverage')
    parser.add_argument('--min-length', type=int, default=500, 
                        help='Minimum character length for articles')
    parser.add_argument('--list-topics', action='store_true', 
                        help='List available topics and exit')
    
    args = parser.parse_args()
    
    if args.list_topics:
        print("Available German Wikipedia topics:")
        for i, topic in enumerate(topics_list[:20]):  # Show first 20
            print(f"  {i+1}. {topic}")
        print(f"  ... and {len(topics_list) - 20} more")
        return
    
    print("🌍 German Wikipedia Scraper for Harper")
    print("=" * 50)
    
    articles = []
    
    if args.topics:
        # Scrape specific topics
        for topic in args.topics:
            print(f"Fetching: {topic}...")
            content = fetch_wikipedia_article(topic)
            if content and len(content) >= args.min_length:
                articles.append((topic, content))
                time.sleep(1)  # Be respectful to Wikipedia servers
            else:
                print(f"❌ Could not fetch or content too short: {topic}")
    else:
        # Scrape random topics
        selected_topics = topics_list[:args.count]
        for topic in selected_topics:
            print(f"Fetching: {topic}...")
            content = fetch_wikipedia_article(topic)
            if content and len(content) >= args.min_length:
                articles.append((topic, content))
                time.sleep(1)  # Be respectful
            else:
                print(f"❌ Could not fetch or content too short: {topic}")
    
    if not articles:
        print("❌ No articles were scraped. Check your internet connection.")
        return
    
    print(f"\n✅ Successfully scraped {len(articles)} articles")
    
    if args.analyze:
        analyze_articles(articles)
    else:
        save_articles(articles, args.output)
        print(f"\n✅ Articles saved to: {args.output}")
        
        # Also provide a summary
        total_chars = sum(len(content) for _, content in articles)
        total_words = sum(len(extract_words(content)) for _, content in articles)
        print(f"📊 Summary: {total_chars} characters, ~{total_words} words across {len(articles)} articles")

if __name__ == '__main__':
    main()