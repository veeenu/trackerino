import pandas as pd, sqlite3

con = sqlite3.connect('data/trackerino.db')
df = pd.read_sql_query('SELECT * from tracking_entries', con)

for i, r in df.iterrows():
    print(r)
    print('')
