#! /usr/bin/python

import sys
import re

ENGLISH = re.compile(rb'[a-zA-Z]+')

def wordcount(text):
    counts = {}
    for word in re.findall(ENGLISH, text):
        word = word.lower()
        if word not in counts:
            counts[word] = 0
        counts[word] += 1

    counts = [(n, word) for (word, n) in counts.items()]
    counts.sort(key=lambda pair: pair[1])
    counts.sort(key=lambda pair: pair[0], reverse=True)
    return counts

if __name__ == '__main__':
    text = open(sys.argv[1], 'rb').read()
    out = open(sys.argv[2], 'wt')
    words = wordcount(text)
    for n, word in wordcount(text):
        print(n, str(word, encoding='ascii'), file=out)
