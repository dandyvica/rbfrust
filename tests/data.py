import string
import random
import sys

# create fake data file to test reader
#recs = ["LL", "NB", "DP", "GL"]
recs = ["LL", "NB", "DP"]
nb_lines = int(sys.argv[1])

# create records for each record ID
rec = {}

rec["LL"] = "LL"
for i,l in enumerate(string.ascii_uppercase[:26]):
    rec["LL"] += l * (i+1)

rec["NB"] = "NB122333444455555666666777777788888888999999999"

rec["DP"] = "DPAAAAABBBBBCCCCCDDDDDEEEEE"

'''for greek_code in range(0x3b1,0x3ca):
    greek_char = chr(greek_code).encode('utf-8')
    print(greek_char)'''

greek = "αβγδεζηθικλμνξοπρστυφχψω"
rec["GL"] = "GL"
for i,l in enumerate(greek):
    rec["GL"] += l * (i+1)

# randomly creates records
for i in range(0,nb_lines):
    # select record ID
    rec_id = random.choice(recs)
    print(rec[rec_id])



