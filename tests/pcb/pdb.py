import sys
import re

# purge description texts
def purge_desc(desc):
    s = desc.strip().rstrip(".")
    return re.sub('\s+', ' ', s)


# 32 - 35       IDcode        rIdCode       ID code of entry that replaced this one. 
re_field = r"^(?P<inf>\d+)[\s\-–]{0,3}(?P<sup>\d*)\s+(?P<data_type>[\w()\d\.]+)\s{2,}(?P<fname>([\w\"\[\]]|\"\w+\")+)(?P<fdesc>.*)"
re_recname = r'.*"(?P<recname>.+)"'
re_offsets = r"^(?P<inf>\d+)[\s\-–]{0,3}(?P<sup>\d*)"

recs = {}

# first, read records
for line in open(sys.argv[2]):
    l = line.strip()
    if l == "": continue

    recname = l[0:15].strip().ljust(6)
    recdesc = l[15:].strip().rstrip(".")

    recs[recname] = purge_desc(recdesc)


for line in open(sys.argv[1]):

    # Header of table give a hint on field positions
    if line.startswith("COLUMNS"):
        cols = [line.index(s) for s in ["COLUMNS", "DATA TYPE", "FIELD", "DEFINITION"]]
        continue

    # new record here
    if "Record name" in line:
        m = re.match(re_recname, line)
        if m:
            print("</record>\n")

            recname = m.group('recname').strip().ljust(6)

            print('<record name="{0}" description="{1}">'.format(recname, recs[recname]))
            print('\t<field name="recName" description="Record name" type="String" start="1" end="6"/>')
            continue
        else:
            print("Error for line: ", line)
            exit(1)

    # field here
    offsets = line[cols[0]:cols[1]]
    m = re.match(re_offsets, line)
    if m:
        # manage single lower offset
        if m.group('sup') == "": 
            inf = m.group('inf')
            sup = inf
        else:
            inf = m.group('inf')
            sup = m.group('sup')    

    data_type = line[cols[1]:cols[2]].strip()

    fname = line[cols[2]:cols[3]].strip()
    if '"' in fname:
        fname = fname.strip().lower()
        fname = re.sub('"', "", fname)

    #fdesc = line[cols[3]:].strip().rstrip(".")
    fdesc = purge_desc(line[cols[3]:])

    #print("<{}><{}><{}><{}>".format(offsets,data_type, fname, fdesc))

    print('\t<field name="{3}" description="{4}" type="{2}" start="{0}" end="{1}"/>'.format(
        inf, sup, data_type, fname, fdesc))
    continue


