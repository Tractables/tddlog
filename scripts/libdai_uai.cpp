// libDAI's junction tree (exact, min-fill) on a UAI network, for `bench.py
// run --libdai`. Usage:  libdai_uai MODEL.uai EVID OUT MAR|PR [I,J,...|@FILE]
// The observations of EVID are applied by slicing the tables. OUT is in
// Merlin's format (scripts/bn_uai_run.py describes it); MAR gives the
// marginals of the listed variables, inline or in FILE (all by default);
// the last line is
// `inference_seconds X`, the junction tree's own time.
#include <dai/alldai.h>
#include <dai/jtree.h>
#include <chrono>
#include <cmath>
#include <cstdio>
#include <fstream>
#include <iostream>
#include <map>
#include <set>
#include <sstream>
#include <vector>

using namespace dai;
using std::size_t;

int main(int argc, char **argv) {
    if (argc < 5) { std::fprintf(stderr, "usage: libdai_uai MODEL EVID OUT MAR|PR [VARS]\n"); return 2; }
    std::string task = argv[4];
    std::ifstream mf(argv[1]);
    std::string kind; size_t n, m;
    mf >> kind >> n;
    std::vector<size_t> sizes(n);
    for (auto &s : sizes) mf >> s;
    mf >> m;
    std::vector<std::vector<size_t>> scopes(m);
    for (auto &sc : scopes) { size_t k; mf >> k; sc.resize(k); for (auto &v : sc) mf >> v; }
    std::vector<std::vector<double>> tables(m);
    for (size_t i = 0; i < m; i++) { size_t e; mf >> e; tables[i].resize(e); for (auto &x : tables[i]) mf >> x; }
    std::map<size_t, size_t> evid;
    { std::ifstream ef(argv[2]); size_t k = 0; ef >> k; for (size_t i = 0; i < k; i++) { size_t v, u; ef >> v >> u; evid[v] = u; } }
    std::vector<size_t> qvars;
    if (argc > 5 && argv[5][0]) {
        std::string listed = argv[5];
        if (listed[0] == '@') {
            std::ifstream vf(listed.substr(1));
            if (!vf) { std::fprintf(stderr, "libdai_uai: cannot read %s\n", listed.c_str() + 1); return 2; }
            std::stringstream all; all << vf.rdbuf(); listed = all.str();
        }
        std::stringstream ss(listed); std::string t;
        while (std::getline(ss, t, ',')) qvars.push_back(std::stoul(t));
    }
    else for (size_t i = 0; i < n; i++) qvars.push_back(i);

    // Slice the observations out; the factors without a variable are constants.
    double log_const = 0.0;
    std::vector<Factor> factors;
    std::set<size_t> present;
    for (size_t i = 0; i < m; i++) {
        const auto &sc = scopes[i];
        std::vector<size_t> rest;
        for (size_t v : sc) if (!evid.count(v)) rest.push_back(v);
        std::vector<Var> vars;
        for (size_t v : rest) vars.push_back(Var(v, sizes[v]));
        VarSet vs(vars.begin(), vars.end(), vars.size());
        Factor f(vs);
        size_t total = tables[i].size();
        for (size_t idx = 0; idx < total; idx++) {
            // the scope's last variable changes fastest
            size_t r = idx; std::vector<size_t> st(sc.size());
            for (size_t j = sc.size(); j-- > 0;) { st[j] = r % sizes[sc[j]]; r /= sizes[sc[j]]; }
            bool keep = true;
            std::map<Var, size_t> state;
            for (size_t j = 0; j < sc.size(); j++) {
                auto it = evid.find(sc[j]);
                if (it != evid.end()) { if (it->second != st[j]) keep = false; }
                else state[Var(sc[j], sizes[sc[j]])] = st[j];
            }
            if (keep) f.set(rest.empty() ? 0 : calcLinearState(vs, state), tables[i][idx]);
        }
        if (rest.empty()) { double c = f[0]; log_const += c > 0 ? std::log(c) : -INFINITY; }
        else { factors.push_back(f); for (size_t v : rest) present.insert(v); }
    }
    std::vector<Var> pv;
    for (size_t v : present) pv.push_back(Var(v, sizes[v]));
    double log_z = 0.0;
    std::map<size_t, std::vector<double>> marg;
    auto t0 = std::chrono::steady_clock::now();
    if (!factors.empty()) {
        FactorGraph fg(factors);
        PropertySet opts;
        opts.set("updates", std::string("HUGIN"));
        opts.set("inference", std::string("SUMPROD"));
        opts.set("heuristic", std::string("MINFILL"));
        JTree jt(fg, opts);
        jt.init();
        jt.run();
        log_z = jt.logZ();
        for (size_t v : qvars) if (present.count(v)) {
            Factor b = jt.belief(Var(v, sizes[v]));
            std::vector<double> row(sizes[v]);
            for (size_t u = 0; u < sizes[v]; u++) row[u] = b[u];
            marg[v] = row;
        }
    }
    double secs = std::chrono::duration<double>(std::chrono::steady_clock::now() - t0).count();
    double lz = log_z + log_const;
    bool ok = std::isfinite(lz);
    std::ofstream out(argv[3]);
    out.precision(12);
    out << "PR\n" << lz << " (" << (ok ? std::exp(lz) : 0.0) << ")\nSTATUS\n" << (ok ? "true" : "false") << "\nMAR\n";
    if (task == "MAR" && ok) {
        out << qvars.size() << "\n";
        for (size_t v : qvars) {
            out << sizes[v];
            for (size_t u = 0; u < sizes[v]; u++) {
                double p = evid.count(v) ? (u == evid[v] ? 1.0 : 0.0) : marg.count(v) ? marg[v][u] : 1.0 / sizes[v];
                out << " " << p;
            }
            out << "\n";
        }
    } else out << "0\n";
    out << "inference_seconds " << secs << "\n";
    return 0;
}
