#!/usr/bin/python3
import sys
import os

# Lire les données POST
content_length = os.environ.get('CONTENT_LENGTH')
if content_length:
    post_data = sys.stdin.read(int(content_length))
    print("Données reçues:", post_data)