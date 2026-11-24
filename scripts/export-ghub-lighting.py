"""Extract lighting from an existing G HUB SQLite backup (never modifies it).

Output stays local: downloaded community effects are not licensed by this repo.
This is preservation of source data, not a promise of compatible playback.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3


def extract(data):
    cards = data.get('cards', {}).get('cards', [])
    lighting_cards = [card for card in cards if any(
        key in card for key in ('softwareLightingSettings', 'firmwareLightingSettings'))]
    ids = {card.get('id') for card in lighting_cards}
    profiles = []
    for profile in data.get('profiles', {}).get('profiles', []):
        assignments = [assignment for assignment in profile.get('assignments', [])
                       if assignment.get('cardId') in ids]
        if assignments or profile.get('lightingCard'):
            profiles.append({key: profile[key] for key in
                             ('id', 'lightingCard', 'activeForApplication') if key in profile}
                            | {'assignments': assignments})
    effects = {key.removeprefix('lighting_effects/'): value for key, value in data.items()
               if key.startswith('lighting_effects/')}
    # Preserve effect cards but omit unrelated profile/account/application data.
    return {'format': 'devhub-ghub-lighting-preservation', 'version': 1,
            'playbackVerified': False,
            'profiles': profiles, 'cards': lighting_cards, 'effects': effects,
            'prefabs': data.get('lighting_prefabs', {})}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('backup', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    if args.output.exists():
        raise SystemExit('Refusing to overwrite existing export')
    with sqlite3.connect(args.backup.resolve().as_uri() + '?mode=ro', uri=True) as conn:
        row = conn.execute('SELECT file FROM data ORDER BY _id DESC LIMIT 1').fetchone()
        if row is None:
            raise SystemExit('No G HUB settings record found')
        data = json.loads(row[0])
    result = extract(data)
    result['sourceSha256'] = hashlib.sha256(args.backup.read_bytes()).hexdigest()
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
    print(f'Preserved {len(result["effects"])} effects and {len(result["cards"])} lighting cards; playback unverified.')


if __name__ == '__main__':
    main()
