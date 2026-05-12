import os
fpath = r'C:\Users\Administrator\.qclaw\workspace\harper-work\quick123-666\harper\harper-core\src\spell\mod.rs'
with open(fpath, 'r', encoding='utf-8') as f:
    content = f.read()

old = 'let mut score = sug.edit_distance as i32 * 10;'
new = 'let base_score = sug.edit_distance as f64 * 10.0;\n    let mut multiplier = 1.0;'

if old in content:
    content = content.replace(old, new, 1)
    with open(fpath, 'w', encoding='utf-8') as f:
        f.write(content)
    print('replaced first line')
else:
    print('NOT FOUND')