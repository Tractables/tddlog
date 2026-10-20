% Biomine (biomine_toy.bmg): the 3 most probable paths between Phenotype_MIM:104300 (Alzheimer disease) and each gene, 8 nodes, 10 edges
% app Gene_EntrezGene:351 gene_entrezgene_351: 3 paths
% apbb2 Gene_EntrezGene:323 gene_entrezgene_323: 3 paths
% bche Gene_EntrezGene:590 gene_entrezgene_590: 3 paths
% pcsk2 Gene_EntrezGene:5126 gene_entrezgene_5126: 3 paths
0.9::edge(article_pubmed_1,gene_entrezgene_590).
0.5::edge(article_pubmed_1,protein_uniprot_p2).
1::edge(gene_entrezgene_323,protein_uniprot_p2).
0.8::edge(gene_entrezgene_351,protein_uniprot_p1).
0.6::edge(gene_entrezgene_351,protein_uniprot_p2).
0.6::edge(gene_entrezgene_5126,gene_entrezgene_590).
0.25::edge(gene_entrezgene_5126,protein_uniprot_p1).
0.9::edge(phenotype_mim_104300,protein_uniprot_p1).
0.5::edge(phenotype_mim_104300,protein_uniprot_p2).
0.82::edge(protein_uniprot_p1,protein_uniprot_p2).
conn(X,Y) :- edge(X,Y).
conn(X,Y) :- edge(Y,X).
path(X,Y) :- conn(X,Y).
path(X,Y) :- conn(X,Z), path(Z,Y).
query(path(phenotype_mim_104300,gene_entrezgene_351)).
query(path(phenotype_mim_104300,gene_entrezgene_323)).
query(path(phenotype_mim_104300,gene_entrezgene_590)).
query(path(phenotype_mim_104300,gene_entrezgene_5126)).
