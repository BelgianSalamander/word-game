
words = []
with open("res/words/lots_of.txt","r") as f:
    words = f.read().split("\n")
    
words2 = []
    
for w in words[:5000]:
    if len(w) < 5: continue
    words2.append(w)

with open("res/words/5000_out.txt", "w") as f:
    f.write("\n".join(words2))