#!/usr/bin/env python3
"""Find gaps in the German rules by injecting errors from treebank gold annotations.

A recall check needs errors at known places. `german_recall_check.py` writes
them with regular expressions, which reach only the constructions someone
thought of. This takes them from the Universal Dependencies treebanks
instead, whose every token carries a lemma, a part of speech, case, number,
person and its head. A token is swapped for **another attested form of the
same lemma** that differs in exactly the feature a class targets, so every
variant is a real word in the wrong place, and the class counts what a
German writer actually gets wrong:

    verb_person   *wir geht*        a finite verb in another person, by the gold pronoun subject
    verb_number   *die Geräte soll*  a finite verb in the other number, by the gold noun subject
    det_case      *mit den Mann*     a determiner in another case, same gender and number
    det_gender    *der Haus*         a singular determiner in another gender
    adj_ending    *ein großen Haus*  an attributive adjective in another case or number
    dat_plural    *mit den Kinder*   the -n of a dative plural taken off
    noun_lower    *das haus*         a common noun written small
    inf_part      *hat lesen*        infinitive and participle swapped

A form that is also attested with the original features is never used, so
*die* for *die* does not count as an error. A variant is detected when a lint
that the original sentence does not have overlaps the swapped token. The
report gives recall per class and, with a class name, per context (which
auxiliary, which preposition, subject before or after the verb), which is
where the gaps show.

    python3 german_treebank_injection.py target/release/harper-cli out.json \
        de_hdt-ud-test.conllu de_pud-ud-test.conllu de_gsd-ud-test.conllu
    python3 german_treebank_injection.py --score out.json [CLASS]

The treebanks are at https://github.com/UniversalDependencies (UD_German-HDT,
-PUD, -GSD). Measured on 1500 variants per class at the sixth round's end:
verb_number 0.6 %, inf_part 0.9 % — the two gaps the seventh round worked on.
"""
import collections, json, os, random, subprocess, sys, tempfile, bisect

random.seed(7)


def parse(path):
    sent = None
    for line in open(path, encoding='utf8'):
        line = line.rstrip('\n')
        if line.startswith('# text = '):
            sent = {'text': line[9:], 'toks': []}
        elif not line and sent:
            if sent['toks']:
                yield sent
            sent = None
        elif line and not line.startswith('#') and sent is not None:
            c = line.split('\t')
            if '-' in c[0] or '.' in c[0]:
                sent['mwt'] = True
                continue
            feats = dict(f.split('=', 1) for f in c[5].split('|')) if c[5] != '_' else {}
            sent['toks'].append(dict(id=int(c[0]), form=c[1], lemma=c[2], upos=c[3], xpos=c[4],
                                     feats=feats, head=int(c[6]), rel=c[7],
                                     space='SpaceAfter=No' not in c[9]))


def score(path, detail=None):
    vs = json.load(open(path))
    c = collections.defaultdict(lambda: [0, 0])
    ctx = collections.defaultdict(lambda: [0, 0])
    for v in vs:
        c[v['cls']][0] += 1; c[v['cls']][1] += bool(v['hits'])
        ctx[(v['cls'], v['ctx'])][0] += 1; ctx[(v['cls'], v['ctx'])][1] += bool(v['hits'])
    for k, (n, h) in sorted(c.items()):
        print(f'{k:12} {h:5}/{n:<5} {100 * h / n:5.1f}%')
    if detail:
        for (k, cx), (n, h) in sorted(ctx.items(), key=lambda kv: -kv[1][0]):
            if k == detail and n >= 8:
                print(f'   {cx:30} {h:4}/{n:<4} {100 * h / n:5.1f}%')


if sys.argv[1] == '--score':
    score(sys.argv[2], sys.argv[3] if len(sys.argv) > 3 else None)
    sys.exit()
sents = [s for f in sys.argv[3:] for s in parse(f) if not s.get('mwt')]
# Rebuild offsets from forms.
for s in sents:
    text, pos = '', []
    for t in s['toks']:
        pos.append((len(text), len(text) + len(t['form'])))
        text += t['form'] + (' ' if t['space'] else '')
    s['text'] = text.strip()
    s['pos'] = pos

inv = collections.defaultdict(collections.Counter)  # (lemma, upos) -> Counter((form, feats tuple))
for s in sents:
    for t in s['toks']:
        inv[(t['lemma'].lower(), t['upos'])][(t['form'], tuple(sorted(t['feats'].items())))] += 1
forms_with = collections.defaultdict(set)  # (lemma,upos,form lower) -> set of feats
for (lemma, upos), c in inv.items():
    for (form, feats), n in c.items():
        forms_with[(lemma, upos, form.lower())].add(feats)


def swap(t, change, keep):
    """Another form of t's lemma whose features differ from t's in the keys of
    `change` (set to those values) and agree on `keep`, and which is never
    attested with t's own features."""
    want = dict(t['feats']); want.update(change)
    cands = collections.Counter()
    for (form, feats), n in inv[(t['lemma'].lower(), t['upos'])].items():
        f = dict(feats)
        if all(f.get(k) == want.get(k) for k in list(change) + keep) and form.lower() != t['form'].lower():
            own = tuple(sorted(t['feats'].items()))
            # the new spelling must not also be a right form here
            if any(all(dict(o).get(k) == t['feats'].get(k) for k in list(change) + keep)
                   for o in forms_with[(t['lemma'].lower(), t['upos'], form.lower())]):
                continue
            cands[form] += n
    if not cands:
        return None
    form = cands.most_common(1)[0][0]
    if t['form'][0].isupper() and form[0].islower():
        form = form[0].upper() + form[1:]
    return form


def children(s, t):
    return [c for c in s['toks'] if c['head'] == t['id']]


def gen(s):
    toks = s['toks']
    out = []
    for i, t in enumerate(toks):
        f = t['feats']
        ctx = ''
        # 1. Verb agreement, by the gold subject.
        if t['upos'] in ('VERB', 'AUX') and f.get('VerbForm') == 'Fin' and 'Person' in f and f.get('Number') in ('Sing','Plur'):
            head = t if t['rel'] not in ('aux', 'cop', 'aux:pass') else next((x for x in toks if x['id'] == t['head']), t)
            subj = [c for c in children(s, head) if c['rel'].startswith('nsubj')]
            if len(subj) == 1:
                sj = subj[0]
                if sj['upos'] == 'PRON' and sj['form'].lower() in ('ich', 'du', 'er', 'wir', 'ihr', 'es', 'man'):
                    new = swap(t, {'Person': {'1': '3', '2': '3', '3': '1'}[f['Person']]}, ['Number', 'Tense', 'Mood'])
                    ctx = 'pron:' + ('V1' if sj['id'] > t['id'] else 'V2') + (':sub' if t['id'] > sj['id'] + 1 and any(c['rel'] == 'mark' for c in children(s, head)) else '')
                    if new: out.append(('verb_person', i, new, ctx))
                elif sj['upos'] in ('NOUN', 'PROPN') and sj['feats'].get('Number') in ('Sing', 'Plur') and not any(c['rel'] == 'conj' for c in children(s, sj)):
                    new = swap(t, {'Number': {'Sing': 'Plur', 'Plur': 'Sing'}[f['Number']]}, ['Person', 'Tense', 'Mood'])
                    ctx = 'noun:' + ('before' if sj['id'] < t['id'] else 'after') + ':' + ('adj' if abs(sj['id'] - t['id']) == 1 else 'far')
                    if new: out.append(('verb_number', i, new, ctx))
        # 2. Determiner case.
        if t['upos'] == 'DET' and 'Case' in f and t['rel'] == 'det':
            head = next((x for x in toks if x['id'] == t['head']), None)
            if head and head['upos'] == 'NOUN':
                prep = [c for c in children(s, head) if c['rel'] == 'case' and c['upos'] == 'ADP']
                ctx = (prep[0]['form'].lower() if prep else 'noprep:' + head['rel']) + ':' + f['Case']
                for case in ('Nom', 'Acc', 'Dat', 'Gen'):
                    if case != f['Case']:
                        new = swap(t, {'Case': case}, ['Gender', 'Number'])
                        if new:
                            out.append(('det_case', i, new, ctx + '>' + case)); break
                for g in ('Masc', 'Fem', 'Neut'):
                    if f.get('Number') == 'Sing' and g != f.get('Gender'):
                        new = swap(t, {'Gender': g}, ['Case', 'Number'])
                        if new:
                            out.append(('det_gender', i, new, t['lemma'] + ':' + f['Case'])); break
        # 3. Adjective ending.
        if t['upos'] == 'ADJ' and 'Case' in f and t['xpos'] == 'ADJA':
            for k, alts in (('Case', ('Nom', 'Acc', 'Dat', 'Gen')), ('Number', ('Sing', 'Plur'))):
                for v in alts:
                    if v != f.get(k):
                        new = swap(t, {k: v}, ['Degree'])
                        if new and new.lower()[:-1] != t['form'].lower()[:-1] or (new and new[-1] != t['form'][-1]):
                            dets = [c for c in children(s, next((x for x in toks if x['id'] == t['head']), t)) if c['upos'] == 'DET']
                            out.append(('adj_ending', i, new, ('det' if dets else 'nodet') + ':' + f.get('Case', '')))
                            break
                else:
                    continue
                break
        # 4. Dative plural -n.
        if t['upos'] == 'NOUN' and f.get('Case') == 'Dat' and f.get('Number') == 'Plur' and t['form'].endswith('n') and not t['form'].endswith(('en', 'ern')) or (
                t['upos'] == 'NOUN' and f.get('Case') == 'Dat' and f.get('Number') == 'Plur' and t['form'].endswith('ern')):
            new = t['form'][:-1]
            if any(dict(o).get('Number') == 'Plur' for o in forms_with[(t['lemma'].lower(), 'NOUN', new.lower())]):
                dets = [c for c in children(s, t) if c['upos'] == 'DET']
                prep = [c for c in children(s, t) if c['rel'] == 'case']
                out.append(('dat_plural', i, new, ('det' if dets else 'nodet') + ':' + (prep[0]['form'].lower() if prep else '-')))
        # 5. Noun capitalization.
        if t['upos'] == 'NOUN' and i > 0 and t['form'][:1].isupper() and t['form'][1:].islower() and len(t['form']) > 3:
            prev = toks[i - 1]
            out.append(('noun_lower', i, t['form'].lower(), prev['upos']))
        # 6. Infinitive vs participle after modal / perfect.
        if t['upos'] == 'VERB' and f.get('VerbForm') in ('Inf', 'Part') and t['xpos'] in ('VVINF', 'VVPP'):
            to = 'Part' if f['VerbForm'] == 'Inf' else 'Inf'
            cands = [form for (form, feats), n in inv[(t['lemma'].lower(), 'VERB')].items() if dict(feats).get('VerbForm') == to and form.lower() != t['form'].lower()]
            if cands:
                new = collections.Counter(cands).most_common(1)[0][0]
                auxs = [c['lemma'] for c in children(s, t) if c['rel'] in ('aux', 'aux:pass')]
                out.append(('inf_part', i, new, f['VerbForm'] + '>' + to + ':' + ','.join(auxs)))
    return out


variants = []
for si, s in enumerate(sents):
    for cls, i, new, ctx in gen(s):
        a, b = s['pos'][i]
        variants.append(dict(cls=cls, ctx=ctx, si=si, start=a, end=a + len(new), old=s['toks'][i]['form'], new=new,
                             text=s['text'][:a] + new + s['text'][b:]))
by = collections.defaultdict(list)
for v in variants:
    by[v['cls']].append(v)
cap = 1500
chosen = []
for cls, vs in by.items():
    random.shuffle(vs)
    chosen += vs[:cap]
print({c: len(v) for c, v in by.items()}, file=sys.stderr)


def lint(texts):
    d = tempfile.mkdtemp()
    buf, offs = '', []
    for t in texts:
        offs.append(len(buf)); buf += t + '\n\n'
    p = d + '/x.md'; open(p, 'w', encoding='utf8').write(buf)
    out = subprocess.run([sys.argv[1], 'lint', '--dialect', 'de', '--format', 'json', p], capture_output=True, text=True).stdout
    per = collections.defaultdict(list)
    for l in (json.loads(out)[0]['lints'] if out.strip() else []):
        s = l['span']['char_start']; i = bisect.bisect_right(offs, s) - 1
        per[i].append((l['rule'], s - offs[i], l['span']['char_end'] - offs[i], l['matched_text']))
    return [per[i] for i in range(len(texts))]


IGN = {'Spaces', 'CommaFixes'}
orig_idx = sorted({v['si'] for v in chosen})
orig_l = dict(zip(orig_idx, lint([sents[i]['text'] for i in orig_idx])))
var_l = lint([v['text'] for v in chosen])
for v, ls in zip(chosen, var_l):
    base = {(r, m) for r, a, b, m in orig_l[v['si']]}
    v['hits'] = [l for l in ls if l[0] not in IGN and l[1] < v['end'] and l[2] > v['start'] and (l[0], l[3]) not in base]
    v['any_new'] = [l for l in ls if l[0] not in IGN and (l[0], l[3]) not in base]
json.dump(chosen, open(sys.argv[2], 'w', encoding='utf8'), ensure_ascii=False)
score(sys.argv[2])
