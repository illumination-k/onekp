# Onekp Downloader

Filtering and Fetching Onekp data from [Gigadb]()

## Show Metadata

Show Liverworts and Mosses metadata

```
onekp metadata --filter-key clade --filter-values Liverworts,Mosses
# --- #
WOGB    Mosses  Andreaeales     Andreaeaceae    Andreaea rupestris      gametophyte, tip of shoots, possibly some developi
ORKS    Mosses  Bartramiales    Bartramiaceae   Philonotis fontana      gametophyte
...
HERT    Liverworts      Sphaerocarpales Sphaerocarpaceae        Sphaerocarpos texanus   gametophyte, possibly some sporophytic tissue
FITN    Liverworts      Treubiales      Treubiaceae     Treubia lacunosa        whole plant
```

## Fetch fasta file

Download protein and cds sequences of Liverworts and Mosses

```
onekp fetch --filter-key clade --filter-values Liverworts,Mosses --sequence-type both --root-dir .
```

## Show Key data

```
onekp show -k clade
```

```
Basal Eudicots
Basalmost angiosperms
Chloranthales
Chromista (Algae)
Conifers
Core Eudicots
Core Eudicots/Asterids
Core Eudicots/Rosids
Cycadales
Dinophyceae
Euglenozoa
Eusporangiate Monilophytes
Ginkgoales
Glaucophyta (Algae)
Gnetales
Green Algae
Hornworts
Leptosporangiate Monilophytes
Liverworts
Lycophytes
Magnoliids
Monocots
Monocots/Commelinids
Mosses
Red Algae
```