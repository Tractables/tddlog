#!/usr/bin/env python3
"""Generate Boolean TPC-H queries over a tuple-independent database.

usage: gen_tpch.py SF [--seed S] [--queries safe|q2,q7,...] [--views]

The probabilistic-database benchmark of SPROUT (Olteanu, Huang, Koch, ICDE
2009) and of dissociation (Gatterbauer and Suciu, VLDB J. 2017): the TPC-H
database with every tuple an independent probabilistic fact, and the TPC-H
queries without their aggregations and inequality joins, as Boolean
queries that keep their selections (SPROUT's "Boolean versions").

The data are written by this script, not by TPC's dbgen (whose license
does not allow redistributing it or its output). It follows the schema and
the value distributions of the TPC-H specification (clause 4.2.3) at scale
factor SF: 10 000 SF suppliers, 200 000 SF parts with four suppliers each
(the specification's formula for PS_SUPPKEY), 150 000 SF customers of
which those whose key is a multiple of three place no order, 1 500 000 SF
orders of one to seven line items, the 25 nations and 5 regions, dates from
1992-01-01 to 1998-12-31 written as integers yyyymmdd. Only the attributes
some query reads are written, and names that queries match with LIKE are
written as the words they match: a part's first name word (`forest%`), the
three syllables of its type, the two of its container.

Each tuple's probability is drawn uniformly from [0.05, 0.95]; those of
lineitem and partsupp, the relations that link the others, are then scaled
by min(1, 100/n) for a relation of n tuples, so that about fifty of them
are expected to hold whatever SF is, as gen_pdb.py scales its S tuples.
Unscaled, every query whose lineage grows with the data would answer 1 to
the last digit at the scale factors the suite uses, and the hard part of
its lineage would not show in the answer. The probabilities change the
answers, not the lineage, whose structure makes a query easy or hard.

Queries (TPC-H numbers; SPROUT's analysis of the Boolean versions):

    hierarchical: q1, q4, q6, q12, q14, q15, q16, q17, q19
    hierarchical under the keys (each order has one customer, each
      supplier and customer one nation): q3, q10, q11, q18
    not hierarchical, the chain supplier - partsupp - part of h0: q2, q20
    not hierarchical, the chain supplier - lineitem - orders (q21) and
      supplier - lineitem - orders - customer (q7)

On TPC-H data the selections of q2 and q20 leave a sparse bipartite graph
between suppliers and parts, while those of q7 and q21 leave one that
densifies as SF grows; `--queries safe` writes every query but q7 and q21.
q18 is the join of customer, orders and lineitem without its HAVING
subquery, and q19 compares the ship mode with 'AIR' and 'AIR REG' as the
specification writes it (dbgen's modes say 'REG AIR', so only AIR matches).

`--views` writes each query as a person would write it over a database,
through views: a selection of one relation, the join of a pair or a
subquery (EXISTS, IN) as a named intermediate predicate, the query's
result relation grouped as the SQL groups it, and the Boolean query as
whether that result is nonempty. Each view only names a part of the
query's join, so the query's lineage, and its answer, are those of the
flat rule; tpch_exact.py, which reads the facts and the query names, is
the reference for both.
"""
import argparse
import datetime
import random

ALL = ['q1', 'q2', 'q3', 'q4', 'q6', 'q7', 'q10', 'q11', 'q12', 'q14', 'q15', 'q16', 'q17', 'q18',
       'q19', 'q20', 'q21']
SAFE = [q for q in ALL if q not in ('q7', 'q21')]

REGIONS = ['africa', 'america', 'asia', 'europe', 'middle_east']
NATIONS = [('algeria', 0), ('argentina', 1), ('brazil', 1), ('canada', 1), ('egypt', 4), ('ethiopia', 0),
           ('france', 3), ('germany', 3), ('india', 2), ('indonesia', 2), ('iran', 4), ('iraq', 4),
           ('japan', 2), ('jordan', 4), ('kenya', 0), ('morocco', 0), ('mozambique', 0), ('peru', 1),
           ('china', 2), ('romania', 3), ('saudi_arabia', 4), ('vietnam', 2), ('russia', 3),
           ('united_kingdom', 3), ('united_states', 1)]
COLORS = ('almond antique aquamarine azure beige bisque black blanched blue blush brown burlywood burnished '
          'chartreuse chiffon chocolate coral cornflower cornsilk cream cyan dark deep dim dodger drab '
          'firebrick floral forest frosted gainsboro ghost goldenrod green grey honeydew hot indian ivory '
          'khaki lace lavender lawn lemon light lime linen magenta maroon medium metallic midnight mint '
          'misty moccasin navajo navy olive orange orchid pale papaya peach peru pink plum powder puff '
          'purple red rose rosy royal saddle salmon sandy seashell sienna sky slate smoke snow spring '
          'steel tan thistle tomato turquoise violet wheat white yellow').split()
TYPE1 = ['standard', 'small', 'medium', 'large', 'economy', 'promo']
TYPE2 = ['anodized', 'burnished', 'plated', 'polished', 'brushed']
TYPE3 = ['tin', 'nickel', 'brass', 'steel', 'copper']
CONTAINER1 = ['sm', 'lg', 'med', 'jumbo', 'wrap']
CONTAINER2 = ['case', 'box', 'bag', 'jar', 'pkg', 'pack', 'can', 'drum']
SEGMENTS = ['automobile', 'building', 'furniture', 'machinery', 'household']
PRIORITIES = ['urgent', 'high', 'medium', 'not_specified', 'low']
INSTRUCTIONS = ['deliver_in_person', 'collect_cod', 'none', 'take_back_return']
MODES = ['reg_air', 'air', 'rail', 'ship', 'truck', 'mail', 'fob']
START, END = datetime.date(1992, 1, 1), datetime.date(1998, 12, 31)
CURRENT = datetime.date(1995, 6, 17)

# The queries, written over the relations below. A date is an integer
# yyyymmdd, so that comparisons on dates are comparisons on integers.
RULES = {
    'q1': ['q1 :- lineitem(_,_,_,_,_,_,_,_,Ship,_,_,_,_), Ship =< 19980902.'],
    'q2': ['q2 :- part(P,_,_,_,_,_,brass,15,_,_), partsupp(P,S), supplier(S,N), nation(N,_,R), region(R,europe).'],
    'q3': ['q3 :- customer(C,_,building), orders(O,C,_,Date,_), Date < 19950315, '
           'lineitem(O,_,_,_,_,_,_,_,Ship,_,_,_,_), Ship > 19950315.'],
    'q4': ['q4 :- orders(O,_,_,Date,_), Date >= 19930701, Date < 19931001, '
           'lineitem(O,_,_,_,_,_,_,_,_,Commit,Receipt,_,_), Commit < Receipt.'],
    'q6': ['q6 :- lineitem(_,_,_,_,Qty,Disc,_,_,Ship,_,_,_,_), Ship >= 19940101, Ship < 19950101, '
           'Disc >= 5, Disc =< 7, Qty < 24.'],
    'q7': ['q7 :- supplier(S,N1), nation(N1,france,_), lineitem(O,_,_,S,_,_,_,_,Ship,_,_,_,_), '
           'Ship >= 19950101, Ship =< 19961231, orders(O,C,_,_,_), customer(C,N2,_), nation(N2,germany,_).',
           'q7 :- supplier(S,N1), nation(N1,germany,_), lineitem(O,_,_,S,_,_,_,_,Ship,_,_,_,_), '
           'Ship >= 19950101, Ship =< 19961231, orders(O,C,_,_,_), customer(C,N2,_), nation(N2,france,_).'],
    'q10': ['q10 :- customer(C,N,_), orders(O,C,_,Date,_), Date >= 19931001, Date < 19940101, '
            'lineitem(O,_,_,_,_,_,r,_,_,_,_,_,_), nation(N,_,_).'],
    'q11': ['q11 :- partsupp(_,S), supplier(S,N), nation(N,germany,_).'],
    'q12': ['q12 :- orders(O,_,_,_,_), lineitem(O,_,_,_,_,_,_,_,Ship,Commit,Receipt,_,Mode), q12_mode(Mode), '
            'Commit < Receipt, Ship < Commit, Receipt >= 19940101, Receipt < 19950101.',
            'q12_mode(mail).', 'q12_mode(ship).'],
    'q14': ['q14 :- lineitem(_,_,P,_,_,_,_,_,Ship,_,_,_,_), Ship >= 19950901, Ship < 19951001, '
            'part(P,_,_,_,_,_,_,_,_,_).'],
    'q15': ['q15 :- supplier(S,_), lineitem(_,_,_,S,_,_,_,_,Ship,_,_,_,_), Ship >= 19960101, Ship < 19960401.'],
    'q16': ['q16 :- partsupp(P,_), part(P,_,_,Brand,medium,polished,_,Size,_,_), Brand \\= brand45, q16_size(Size).']
           + [f'q16_size({s}).' for s in (49, 14, 23, 45, 19, 3, 36, 9)],
    'q17': ['q17 :- lineitem(_,_,P,_,_,_,_,_,_,_,_,_,_), part(P,_,_,brand23,_,_,_,_,med,box).'],
    'q18': ['q18 :- customer(C,_,_), orders(O,C,_,_,_), lineitem(O,_,_,_,_,_,_,_,_,_,_,_,_).'],
    'q19': ['q19 :- lineitem(_,_,P,_,Qty,_,_,_,_,_,_,deliver_in_person,Mode), q19_mode(Mode), '
            'part(P,_,_,Brand,_,_,_,Size,C1,C2), q19_case(Brand,C1,C2,QLo,QHi,SHi), '
            'Qty >= QLo, Qty =< QHi, Size >= 1, Size =< SHi.',
            'q19_mode(air).', "q19_mode('air reg')."]
           + [f'q19_case(brand12,sm,{c},1,11,5).' for c in ('case', 'box', 'pack', 'pkg')]
           + [f'q19_case(brand23,med,{c},10,20,10).' for c in ('bag', 'box', 'pkg', 'pack')]
           + [f'q19_case(brand34,lg,{c},20,30,15).' for c in ('case', 'box', 'pack', 'pkg')],
    'q20': ['q20 :- supplier(S,N), nation(N,canada,_), partsupp(P,S), part(P,forest,_,_,_,_,_,_,_,_).'],
    'q21': ['q21 :- supplier(S,N), nation(N,saudi_arabia,_), lineitem(O,_,_,S,_,_,_,_,_,Commit,Receipt,_,_), '
            'Receipt > Commit, orders(O,_,f,_,_).'],
}
# The queries through views (`--views`). A view one query defines with the
# same rules as another (customer_nation in q7 and q10) is written once.
VIEWS = {
    'q1': ['shipped(O,L) :- lineitem(O,L,_,_,_,_,_,_,Ship,_,_,_,_), Ship =< 19980902.',
           'q1 :- shipped(_,_).'],
    'q2': ['european_nation(N) :- nation(N,_,R), region(R,europe).',
           'european_supplier(S) :- supplier(S,N), european_nation(N).',
           'brass_part(P) :- part(P,_,_,_,_,_,brass,15,_,_).',
           'q2_part(P) :- brass_part(P), partsupp(P,S), european_supplier(S).',
           'q2 :- q2_part(_).'],
    'q3': ['building_customer(C) :- customer(C,_,building).',
           'early_order(O,C) :- orders(O,C,_,Date,_), Date < 19950315.',
           'shipped_late(O) :- lineitem(O,_,_,_,_,_,_,_,Ship,_,_,_,_), Ship > 19950315.',
           'q3_order(O) :- building_customer(C), early_order(O,C), shipped_late(O).',
           'q3 :- q3_order(_).'],
    'q4': ['late_lineitem(O) :- lineitem(O,_,_,_,_,_,_,_,_,Commit,Receipt,_,_), Commit < Receipt.',
           'q4_priority(Pri) :- orders(O,_,_,Date,Pri), Date >= 19930701, Date < 19931001, late_lineitem(O).',
           'q4 :- q4_priority(_).'],
    'q6': ['discounted(O,L) :- lineitem(O,L,_,_,Qty,Disc,_,_,Ship,_,_,_,_), Ship >= 19940101, Ship < 19950101, '
           'Disc >= 5, Disc =< 7, Qty < 24.',
           'q6 :- discounted(_,_).'],
    'q7': ['supplier_nation(S,Name) :- supplier(S,N), nation(N,Name,_).',
           'customer_nation(C,Name) :- customer(C,N,_), nation(N,Name,_).',
           'shipping(S,C) :- lineitem(O,_,_,S,_,_,_,_,Ship,_,_,_,_), Ship >= 19950101, Ship =< 19961231, '
           'orders(O,C,_,_,_).',
           'q7_volume(N1,N2) :- q7_pair(N1,N2), supplier_nation(S,N1), shipping(S,C), customer_nation(C,N2).',
           'q7 :- q7_volume(_,_).',
           'q7_pair(france,germany).', 'q7_pair(germany,france).'],
    'q10': ['returned_order(O) :- lineitem(O,_,_,_,_,_,r,_,_,_,_,_,_).',
            'customer_nation(C,Name) :- customer(C,N,_), nation(N,Name,_).',
            'q10_customer(C) :- orders(O,C,_,Date,_), Date >= 19931001, Date < 19940101, returned_order(O), '
            'customer_nation(C,_).',
            'q10 :- q10_customer(_).'],
    'q11': ['german_supplier(S) :- supplier(S,N), nation(N,germany,_).',
            'q11_part(P) :- partsupp(P,S), german_supplier(S).',
            'q11 :- q11_part(_).'],
    'q12': ['q12_lineitem(O,Mode) :- lineitem(O,_,_,_,_,_,_,_,Ship,Commit,Receipt,_,Mode), q12_mode(Mode), '
            'Commit < Receipt, Ship < Commit, Receipt >= 19940101, Receipt < 19950101.',
            'q12_shipmode(Mode) :- orders(O,_,_,_,_), q12_lineitem(O,Mode).',
            'q12 :- q12_shipmode(_).',
            'q12_mode(mail).', 'q12_mode(ship).'],
    'q14': ['shipped_in_september(P) :- lineitem(_,_,P,_,_,_,_,_,Ship,_,_,_,_), Ship >= 19950901, Ship < 19951001.',
            'q14 :- shipped_in_september(P), part(P,_,_,_,_,_,_,_,_,_).'],
    'q15': ['revenue(S) :- lineitem(_,_,_,S,_,_,_,_,Ship,_,_,_,_), Ship >= 19960101, Ship < 19960401.',
            'q15 :- supplier(S,_), revenue(S).'],
    'q16': ['q16_part(P,Brand,Size) :- part(P,_,_,Brand,medium,polished,_,Size,_,_), Brand \\= brand45, '
            'q16_size(Size).',
            'q16_group(Brand,Size) :- partsupp(P,_), q16_part(P,Brand,Size).',
            'q16 :- q16_group(_,_).']
           + [f'q16_size({s}).' for s in (49, 14, 23, 45, 19, 3, 36, 9)],
    'q17': ['q17_part(P) :- part(P,_,_,brand23,_,_,_,_,med,box).',
            'q17 :- lineitem(_,_,P,_,_,_,_,_,_,_,_,_,_), q17_part(P).'],
    'q18': ['big_order(O) :- lineitem(O,_,_,_,_,_,_,_,_,_,_,_,_).',
            'customer_order(O,C) :- orders(O,C,_,_,_), big_order(O).',
            'buyer(C) :- customer(C,_,_), customer_order(_,C).',
            'q18 :- buyer(_).'],
    'q19': ['q19_lineitem(P,Qty) :- lineitem(_,_,P,_,Qty,_,_,_,_,_,_,deliver_in_person,Mode), q19_mode(Mode).',
            'q19_part(P,QLo,QHi) :- part(P,_,_,Brand,_,_,_,Size,C1,C2), q19_case(Brand,C1,C2,QLo,QHi,SHi), '
            'Size >= 1, Size =< SHi.',
            'q19 :- q19_lineitem(P,Qty), q19_part(P,QLo,QHi), Qty >= QLo, Qty =< QHi.']
           + [r for r in RULES['q19'] if not r.startswith('q19 :-')],
    'q20': ['forest_part(P) :- part(P,forest,_,_,_,_,_,_,_,_).',
            'forest_supplier(S) :- partsupp(P,S), forest_part(P).',
            'q20_supplier(S) :- supplier(S,N), nation(N,canada,_), forest_supplier(S).',
            'q20 :- q20_supplier(_).'],
    'q21': ['saudi_supplier(S) :- supplier(S,N), nation(N,saudi_arabia,_).',
            'late_delivery(S,O) :- lineitem(O,_,_,S,_,_,_,_,_,Commit,Receipt,_,_), Receipt > Commit.',
            'finished_order(O) :- orders(O,_,f,_,_).',
            'waiting_supplier(S) :- saudi_supplier(S), late_delivery(S,O), finished_order(O).',
            'q21 :- waiting_supplier(_).'],
}
# The relations each query reads, so that a program holds only those.
READS = {
    'q1': {'lineitem'}, 'q2': {'part', 'partsupp', 'supplier', 'nation', 'region'},
    'q3': {'customer', 'orders', 'lineitem'}, 'q4': {'orders', 'lineitem'}, 'q6': {'lineitem'},
    'q7': {'supplier', 'nation', 'lineitem', 'orders', 'customer'},
    'q10': {'customer', 'orders', 'lineitem', 'nation'}, 'q11': {'partsupp', 'supplier', 'nation'},
    'q12': {'orders', 'lineitem'}, 'q14': {'lineitem', 'part'}, 'q15': {'supplier', 'lineitem'},
    'q16': {'partsupp', 'part'}, 'q17': {'lineitem', 'part'}, 'q18': {'customer', 'orders', 'lineitem'},
    'q19': {'lineitem', 'part'}, 'q20': {'supplier', 'nation', 'partsupp', 'part'},
    'q21': {'supplier', 'nation', 'lineitem', 'orders'},
}


def ymd(d):
    return d.year * 10000 + d.month * 100 + d.day


def suppkey(partkey, i, s):
    """The i-th supplier of a part (TPC-H 4.2.3, PS_SUPPKEY), keys from 1."""
    return (partkey + i * (s // 4 + (partkey - 1) // s)) % s + 1


def database(sf, rng):
    """The relations at scale factor `sf`, each a list of tuples."""
    n_supp, n_part = max(1, round(10000 * sf)), max(1, round(200000 * sf))
    n_cust, n_ord = max(1, round(150000 * sf)), max(1, round(1500000 * sf))
    db = {'region': [(r, name) for r, name in enumerate(REGIONS)],
          'nation': [(n, name, r) for n, (name, r) in enumerate(NATIONS)],
          'supplier': [(s, rng.randrange(25)) for s in range(1, n_supp + 1)],
          'customer': [(c, rng.randrange(25), rng.choice(SEGMENTS)) for c in range(1, n_cust + 1)],
          'part': [], 'partsupp': [], 'orders': [], 'lineitem': []}
    for p in range(1, n_part + 1):
        m = rng.randint(1, 5)
        db['part'].append((p, rng.sample(COLORS, 5)[0], f'manufacturer{m}', f'brand{m}{rng.randint(1, 5)}',
                           rng.choice(TYPE1), rng.choice(TYPE2), rng.choice(TYPE3), rng.randint(1, 50),
                           rng.choice(CONTAINER1), rng.choice(CONTAINER2)))
        # The formula can give a part the same supplier twice when there
        # are few suppliers (SF 0.001's ten); (part, supplier) is a key.
        for s in dict.fromkeys(suppkey(p, i, n_supp) for i in range(4)):
            db['partsupp'].append((p, s))
    buyers = [c for c in range(1, n_cust + 1) if c % 3 != 0] or [1]
    span = (END - START).days - 151
    for o in range(1, n_ord + 1):
        date = START + datetime.timedelta(days=rng.randint(0, span))
        items = []
        for line in range(1, rng.randint(1, 7) + 1):
            p = rng.randint(1, n_part)
            ship = date + datetime.timedelta(days=rng.randint(1, 121))
            commit = date + datetime.timedelta(days=rng.randint(30, 90))
            receipt = ship + datetime.timedelta(days=rng.randint(1, 30))
            flag = rng.choice('ra') if receipt <= CURRENT else 'n'
            status = 'o' if ship > CURRENT else 'f'
            items.append((o, line, p, suppkey(p, rng.randrange(4), n_supp), rng.randint(1, 50),
                          rng.randint(0, 10), flag, status, ymd(ship), ymd(commit), ymd(receipt),
                          rng.choice(INSTRUCTIONS), rng.choice(MODES)))
        statuses = {it[7] for it in items}
        status = 'f' if statuses == {'f'} else 'o' if statuses == {'o'} else 'p'
        db['orders'].append((o, rng.choice(buyers), status, ymd(date), rng.choice(PRIORITIES)))
        db['lineitem'] += items
    return db


# The relations whose probabilities are scaled (docstring), and the number
# of their tuples that are expected to hold, times two.
LINKS = {'lineitem', 'partsupp'}
EXPECTED = 100


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('sf', type=float)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--queries', default='safe', help='safe (all but q7 and q21), all, or a comma-separated list')
    ap.add_argument('--views', action='store_true', help='each query through views (intermediate predicates)')
    a = ap.parse_args()
    qs = SAFE if a.queries == 'safe' else ALL if a.queries == 'all' else a.queries.split(',')
    unknown = [q for q in qs if q not in RULES]
    if unknown:
        ap.error('unknown queries: ' + ', '.join(unknown))
    if a.sf <= 0:
        ap.error('SF must be positive')
    rng = random.Random(a.seed)
    db = database(a.sf, rng)
    read = set().union(*(READS[q] for q in qs))
    views = ', through views' if a.views else ''
    print(f'% TPC-H, tuple-independent: SF {a.sf}, seed {a.seed}, queries {",".join(qs)}{views}')
    for rel in ['region', 'nation', 'supplier', 'customer', 'part', 'partsupp', 'orders', 'lineitem']:
        # Every relation's probabilities are drawn, read or not, so that a
        # relation's tuples have the same probabilities whichever queries
        # the program holds.
        scale = min(1.0, EXPECTED / len(db[rel])) if rel in LINKS else 1.0
        # Nine decimals, written without an exponent.
        probs = [f'{rng.uniform(0.05, 0.95) * scale:.9f}'.rstrip('0') for _ in db[rel]]
        if rel not in read:
            continue
        for p, t in zip(probs, db[rel]):
            print(f"{p}::{rel}({','.join(map(str, t))}).")
    rules = VIEWS if a.views else RULES
    for rule in dict.fromkeys(r for q in qs for r in rules[q]):
        print(rule)
    for q in qs:
        print(f'query({q}).')


if __name__ == '__main__':
    main()
