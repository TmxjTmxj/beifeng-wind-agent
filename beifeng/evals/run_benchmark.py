#!/usr/bin/env python3
"""BeiFeng Wind O&M Agent — Benchmark Evaluation Runner

Reads benchmark.json, queries the RAG HTTP service for each test case,
scores results against expected values, and produces a quantitative report.

Usage:
  1. Start the RAG service first:
     CLAW_RAG_MOCK_PROVIDERS=1 cargo run -p claw-rag-service -- serve \
       --db ./beifeng/data/wind.sqlite \
       --graph ./beifeng/knowledge/knowledge_graph/wind_fault_graph.json
  2. Run this script:
     python beifeng/evals/run_benchmark.py [--url http://127.0.0.1:8787]
"""

import json
import os
import sys
import time
import subprocess
import urllib.request
import urllib.error
from pathlib import Path
from collections import defaultdict

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
BENCHMARK_PATH = Path(__file__).resolve().parent / "benchmark.json"
DEFAULT_URL = "http://127.0.0.1:8787"
QUERY_ENDPOINT = "/v1/query"
HEALTH_ENDPOINT = "/health"

# ---------------------------------------------------------------------------
# Scoring helpers
# ---------------------------------------------------------------------------

def score_component_inference(resp: dict, expected: dict) -> dict:
    """Score whether the inferred component matches expected."""
    result = {"max_score": 1, "score": 0, "details": ""}
    expected_comp = expected.get("expected_component", "")
    if not expected_comp:
        result["details"] = "No expected component specified"
        return result

    # Check graph_suggestions first (most reliable)
    graph_suggestions = resp.get("graph_suggestions", [])
    matched_components = set()
    for gs in graph_suggestions:
        comp = gs.get("component", "")
        if comp:
            matched_components.add(comp)

    # Also check advice problem_summary
    advice = resp.get("advice", {})
    advice_text = json.dumps(advice, ensure_ascii=False)

    # Check hit domains
    hits = resp.get("hits", [])
    hit_domains = set()
    for h in hits:
        domain = h.get("domain", "")
        if domain:
            hit_domains.add(domain)

    # Scoring: exact match in graph_suggestions = 1.0, in hit domains = 0.7, partial = 0.5
    if expected_comp in matched_components:
        result["score"] = 1
        result["details"] = f"Exact match in graph_suggestions: {expected_comp}"
    elif expected_comp in hit_domains:
        result["score"] = 0.7
        result["details"] = f"Match in hit domains: {expected_comp}, graph matched: {matched_components}"
    elif expected_comp.lower() in advice_text.lower():
        result["score"] = 0.5
        result["details"] = f"Partial match in advice text, graph: {matched_components}, domains: {hit_domains}"
    else:
        result["details"] = f"NOT matched. Expected: {expected_comp}, got graph: {matched_components}, domains: {hit_domains}"

    return result


def score_graph_matching(resp: dict, expected: dict) -> dict:
    """Score whether expected graph entries are returned."""
    expected_hits = expected.get("expected_graph_hits", [])
    result = {"max_score": len(expected_hits) if expected_hits else 0, "score": 0, "details": ""}

    if not expected_hits:
        result["details"] = "No expected graph hits"
        return result

    graph_suggestions = resp.get("graph_suggestions", [])
    returned_ids = {gs.get("entry_id", "") for gs in graph_suggestions}

    matched = 0
    missing = []
    for eh in expected_hits:
        if eh in returned_ids:
            matched += 1
        else:
            missing.append(eh)

    result["score"] = matched
    if missing:
        result["details"] = f"Matched {matched}/{len(expected_hits)}, missing: {missing}, returned: {returned_ids}"
    else:
        result["details"] = f"All {matched}/{len(expected_hits)} matched. Returned IDs: {returned_ids}"

    return result


def score_rag_recall(resp: dict, expected: dict) -> dict:
    """Score RAG retrieval quality: top hit score and relevance threshold."""
    threshold = expected.get("relevance_threshold", 0.5)
    hits = resp.get("hits", [])

    result = {"max_score": 3, "score": 0, "details": ""}

    if not hits:
        result["details"] = "No hits returned"
        return result

    top_score = hits[0].get("score_breakdown", {}).get("final_score", 0)
    scores_above_threshold = sum(1 for h in hits if h.get("score_breakdown", {}).get("final_score", 0) >= threshold)

    # Score: 1 point if top hit above threshold, 1 point if 3+ hits above, 1 point if 5+ hits
    if top_score >= threshold:
        result["score"] += 1
    if scores_above_threshold >= 3:
        result["score"] += 1
    if scores_above_threshold >= 5:
        result["score"] += 1

    result["details"] = (
        f"Top score: {top_score:.3f} (threshold: {threshold}), "
        f"hits above threshold: {scores_above_threshold}/{len(hits)}"
    )
    return result


def score_advice_consistency(resp: dict, expected: dict) -> dict:
    """Score whether advice contains expected keywords."""
    expected_keywords = expected.get("expected_advice_keywords", [])
    result = {"max_score": len(expected_keywords) if expected_keywords else 0, "score": 0, "details": ""}

    if not expected_keywords:
        result["details"] = "No expected keywords"
        return result

    # Collect all advice text
    advice = resp.get("advice", {})
    advice_text = json.dumps(advice, ensure_ascii=False).lower()
    risk = resp.get("risk_assessment", {})
    risk_text = json.dumps(risk, ensure_ascii=False).lower()
    graph_suggestions = resp.get("graph_suggestions", [])
    graph_text = json.dumps(graph_suggestions, ensure_ascii=False).lower()

    all_text = advice_text + " " + risk_text + " " + graph_text

    matched = 0
    missing = []
    for kw in expected_keywords:
        if kw.lower() in all_text:
            matched += 1
        else:
            missing.append(kw)

    result["score"] = matched
    if missing:
        result["details"] = f"Matched {matched}/{len(expected_keywords)}, missing keywords: {missing}"
    else:
        result["details"] = f"All {matched}/{len(expected_keywords)} keywords found"

    return result


def score_safety_compliance(resp: dict, expected: dict) -> dict:
    """Score safety compliance: forbidden actions and human confirmation."""
    safety_trigger = expected.get("safety_trigger", False)
    result = {"max_score": 2 if safety_trigger else 1, "score": 0, "details": ""}

    advice = resp.get("advice", {})
    risk = resp.get("risk_assessment", {})
    advice_text = json.dumps(advice, ensure_ascii=False).lower()
    risk_text = json.dumps(risk, ensure_ascii=False).lower()

    # Check if safety_notes exist in advice
    safety_notes = advice.get("safety_notes", [])
    forbidden_actions = risk.get("forbidden_actions", [])
    human_required = risk.get("human_confirmation_required", False)

    # Score: 1 point if safety_notes or forbidden_actions are present
    if safety_notes or forbidden_actions:
        result["score"] += 1
        result["details"] += f"Safety notes: {len(safety_notes)}, Forbidden actions: {len(forbidden_actions)}. "
    else:
        result["details"] += "No safety notes or forbidden actions found. "

    # If safety_trigger, also check for human confirmation
    if safety_trigger:
        if human_required:
            result["score"] += 1
            result["details"] += "Human confirmation required: YES. "
        else:
            result["details"] += "Human confirmation required: NO (expected YES). "

    # Also check expected advice keywords related to safety
    expected_keywords = expected.get("expected_advice_keywords", [])
    safety_keywords_found = 0
    all_text = advice_text + " " + risk_text
    for kw in expected_keywords:
        if kw.lower() in all_text:
            safety_keywords_found += 1

    if expected_keywords:
        result["details"] += f"Safety keywords: {safety_keywords_found}/{len(expected_keywords)}"

    return result


def score_risk_assessment(resp: dict, expected: dict) -> dict:
    """Score risk level accuracy."""
    expected_risk = expected.get("expected_risk", "")
    result = {"max_score": 1, "score": 0, "details": ""}

    if not expected_risk:
        result["details"] = "No expected risk level"
        return result

    risk = resp.get("risk_assessment", {})
    advice = resp.get("advice", {})
    actual_risk = risk.get("risk_level", advice.get("risk_level", "Unknown"))

    if actual_risk == expected_risk:
        result["score"] = 1
        result["details"] = f"Risk level matches: {actual_risk}"
    elif (actual_risk == "High" and expected_risk == "Critical") or \
         (actual_risk == "Critical" and expected_risk == "High"):
        result["score"] = 0.5
        result["details"] = f"Risk level close: expected {expected_risk}, got {actual_risk}"
    else:
        result["details"] = f"Risk level mismatch: expected {expected_risk}, got {actual_risk}"

    return result


def score_report_generation(resp: dict, expected: dict) -> dict:
    """Score report generation: check for key report sections in advice."""
    expected_keywords = expected.get("expected_advice_keywords", [])
    result = {"max_score": max(len(expected_keywords), 1), "score": 0, "details": ""}

    advice = resp.get("advice", {})
    advice_text = json.dumps(advice, ensure_ascii=False).lower()

    matched = 0
    for kw in expected_keywords:
        if kw.lower() in advice_text:
            matched += 1

    result["score"] = matched
    result["details"] = f"Report keywords: {matched}/{len(expected_keywords)}"

    return result


def _response_text(resp: dict) -> str:
    """Collect searchable response text for composite benchmark dimensions."""
    return json.dumps(
        {
            "hits": resp.get("hits", []),
            "graph_suggestions": resp.get("graph_suggestions", []),
            "advice": resp.get("advice", {}),
            "risk_assessment": resp.get("risk_assessment", {}),
        },
        ensure_ascii=False,
    ).lower()


def score_multi_component(resp: dict, expected: dict) -> dict:
    """Score composite faults that mention more than one subsystem."""
    expected_components = expected.get("expected_components") or [expected.get("expected_component")]
    expected_components = [c for c in expected_components if c]
    expected_keywords = expected.get("expected_advice_keywords", [])
    result = {
        "max_score": len(expected_components) + len(expected_keywords),
        "score": 0,
        "details": "",
    }

    text = _response_text(resp)
    graph_components = {
        gs.get("component", "").lower()
        for gs in resp.get("graph_suggestions", [])
        if gs.get("component")
    }

    missing_components = []
    for component in expected_components:
        if component.lower() in graph_components or component.lower() in text:
            result["score"] += 1
        else:
            missing_components.append(component)

    missing_keywords = []
    for kw in expected_keywords:
        if kw.lower() in text:
            result["score"] += 1
        else:
            missing_keywords.append(kw)

    result["details"] = (
        f"Components missing: {missing_components or 'none'}, "
        f"keywords missing: {missing_keywords or 'none'}"
    )
    return result


def score_historical_context(resp: dict, expected: dict) -> dict:
    """Score whether the answer uses maintenance/fault history language."""
    expected_keywords = expected.get("expected_advice_keywords", [])
    result = {"max_score": max(len(expected_keywords), 1), "score": 0, "details": ""}

    text = _response_text(resp)
    matched = 0
    missing = []
    for kw in expected_keywords:
        if kw.lower() in text:
            matched += 1
        else:
            missing.append(kw)

    result["score"] = matched
    result["details"] = f"Historical keywords: {matched}/{len(expected_keywords)}, missing: {missing}"
    return result


def score_scada_derived(resp: dict, expected: dict) -> dict:
    """Score SCADA-parameter questions using component inference plus advice keywords."""
    component_score = score_component_inference(resp, expected)
    advice_score = score_advice_consistency(resp, expected)
    result = {
        "max_score": component_score["max_score"] + advice_score["max_score"],
        "score": component_score["score"] + advice_score["score"],
        "details": f"{component_score['details']} | {advice_score['details']}",
    }
    return result


# Category -> scorer mapping
SCORERS = {
    "component_inference": score_component_inference,
    "graph_matching": score_graph_matching,
    "rag_recall": score_rag_recall,
    "advice_consistency": score_advice_consistency,
    "safety_compliance": score_safety_compliance,
    "risk_assessment": score_risk_assessment,
    "report_generation": score_report_generation,
    "multi_component": score_multi_component,
    "historical_context": score_historical_context,
    "scada_derived": score_scada_derived,
}

# ---------------------------------------------------------------------------
# Report generation via CLI
# ---------------------------------------------------------------------------

def generate_report_via_cli(query: str, component: str, symptom: str) -> str | None:
    """Run claw-rag-service report CLI command and return the markdown output."""
    binary_path = Path(__file__).resolve().parent.parent.parent / "rust" / "target" / "release" / "claw-rag-service"
    if not binary_path.exists():
        binary_path = binary_path.with_suffix(".exe")
    if not binary_path.exists():
        binary_path = Path(__file__).resolve().parent.parent.parent / "rust" / "target" / "debug" / "claw-rag-service"
    if not binary_path.exists():
        binary_path = binary_path.with_suffix(".exe")
    if not binary_path.exists():
        return None

    cmd = [
        str(binary_path), "report-generate",
        "--problem", query,
        "--component", component,
        "--symptom", symptom,
        "--db", str(Path(__file__).resolve().parent.parent / "data" / "wind.sqlite"),
        "--graph", str(Path(__file__).resolve().parent.parent / "knowledge" / "knowledge_graph" / "wind_fault_graph.json"),
    ]
    try:
        result = subprocess.run(cmd, capture_output=True, timeout=60,
                              env={**os.environ, "CLAW_RAG_MOCK_PROVIDERS": "1"},
                              encoding="utf-8", errors="replace")
        if result.returncode == 0:
            return result.stdout
        else:
            print(f"  Report CLI error: {result.stderr[:200]}")
            return None
    except Exception as e:
        print(f"  Report CLI failed: {e}")
        return None


# Report sections expected in standard wind report
REPORT_SECTIONS = [
    "基本信息", "故障描述", "故障原因分析", "检测建议",
    "风险评估", "维护建议", "安全注意事项", "证据来源",
    "缺失数据", "建议复查周期", "置信度",
]


def score_report_cli(report_md: str, expected: dict) -> dict:
    """Score report generation quality by checking for standard sections."""
    expected_sections = expected.get("expected_report_sections", REPORT_SECTIONS)
    result = {"max_score": len(expected_sections), "score": 0, "details": ""}

    if not report_md:
        result["details"] = "No report generated"
        return result

    report_lower = report_md.lower()
    matched = 0
    missing = []
    for section in expected_sections:
        if section.lower() in report_lower:
            matched += 1
        else:
            missing.append(section)

    result["score"] = matched
    if missing:
        result["details"] = f"Sections found: {matched}/{len(expected_sections)}, missing: {missing}"
    else:
        result["details"] = f"All {matched}/{len(expected_sections)} sections found"

    return result

# ---------------------------------------------------------------------------
# Query the RAG service
# ---------------------------------------------------------------------------

def query_rag(base_url: str, question: dict) -> dict | None:
    """Send a query to the RAG HTTP service and return the JSON response."""
    url = base_url.rstrip("/") + QUERY_ENDPOINT
    payload = json.dumps({
        "query": question["query"],
        "top_k": 8,
        "search_mode": "hybrid",
    }).encode("utf-8")

    req = urllib.request.Request(
        url,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.URLError as e:
        print(f"  ERROR: Failed to query RAG service: {e}")
        return None
    except Exception as e:
        print(f"  ERROR: Unexpected error: {e}")
        return None


def check_health(base_url: str) -> bool:
    """Check if the RAG service is running."""
    url = base_url.rstrip("/") + HEALTH_ENDPOINT
    try:
        with urllib.request.urlopen(url, timeout=5) as resp:
            return resp.read().decode().strip() == "ok"
    except Exception:
        return False

# ---------------------------------------------------------------------------
# Main evaluation loop
# ---------------------------------------------------------------------------

def run_evaluation(base_url: str, benchmark: dict, enable_report: bool = False) -> dict:
    """Run all benchmark questions and collect scores."""
    questions = benchmark.get("questions", [])
    results = []

    # Per-category accumulators
    category_scores = defaultdict(lambda: {"earned": 0, "max": 0})
    total_earned = 0
    total_max = 0

    for i, q in enumerate(questions):
        qid = q["id"]
        category = q["category"]
        print(f"[{i+1}/{len(questions)}] {qid} ({category}): {q['query'][:40]}...")

        # Handle report_generation via CLI if enabled
        if category == "report_generation" and enable_report:
            component = q.get("component", q.get("expected_component", ""))
            symptom = q.get("symptom", "")
            report_md = generate_report_via_cli(q["query"], component, symptom)
            score_result = score_report_cli(report_md, q)

            all_scores = {category: score_result}

            for cat, sr in all_scores.items():
                category_scores[cat]["earned"] += sr["score"]
                category_scores[cat]["max"] += sr["max_score"]
                total_earned += sr["score"]
                total_max += sr["max_score"]

            results.append({
                "id": qid,
                "category": category,
                "query": q["query"],
                "scores": all_scores,
            })
            continue

        resp = query_rag(base_url, q)
        if resp is None:
            results.append({
                "id": qid,
                "category": category,
                "query": q["query"],
                "error": "Failed to query RAG service",
                "scores": {},
            })
            continue

        # Run the scorer for this category
        scorer = SCORERS.get(category)
        score_result = scorer(resp, q) if scorer else {"max_score": 0, "score": 0, "details": "No scorer"}

        # Also run cross-cutting scorers based on what's expected
        all_scores = {category: score_result}

        # Always score risk if expected_risk is present
        if q.get("expected_risk") and category != "risk_assessment":
            risk_score = score_risk_assessment(resp, q)
            all_scores["risk_assessment"] = risk_score

        # Always score advice keywords if present
        if q.get("expected_advice_keywords") and category not in ("advice_consistency", "safety_compliance", "report_generation"):
            advice_score = score_advice_consistency(resp, q)
            all_scores["advice_consistency"] = advice_score

        # Accumulate
        for cat, sr in all_scores.items():
            category_scores[cat]["earned"] += sr["score"]
            category_scores[cat]["max"] += sr["max_score"]
            total_earned += sr["score"]
            total_max += sr["max_score"]

        results.append({
            "id": qid,
            "category": category,
            "query": q["query"],
            "scores": all_scores,
            "top_hit_score": resp.get("hits", [{}])[0].get("score_breakdown", {}).get("final_score", 0) if resp.get("hits") else 0,
            "graph_suggestions_count": len(resp.get("graph_suggestions", [])),
            "safety_notes_count": len(resp.get("advice", {}).get("safety_notes", [])),
            "forbidden_actions_count": len(resp.get("risk_assessment", {}).get("forbidden_actions", [])),
        })

    return {
        "results": results,
        "category_scores": dict(category_scores),
        "total_earned": total_earned,
        "total_max": total_max,
        "overall_pct": round(total_earned / total_max * 100, 1) if total_max > 0 else 0,
    }


def format_report(eval_result: dict, benchmark: dict) -> str:
    """Format the evaluation result as a Markdown report."""
    lines = []
    lines.append("# BeiFeng Wind O&M Agent — Benchmark Evaluation Report")
    lines.append("")
    lines.append(f"**Date**: {time.strftime('%Y-%m-%d %H:%M')}")
    lines.append(f"**Questions**: {len(benchmark.get('questions', []))}")
    lines.append(f"**Dimensions**: {', '.join(benchmark.get('dimensions', []))}")
    lines.append("")

    # Overall score
    lines.append("## Overall Score")
    lines.append("")
    pct = eval_result["overall_pct"]
    emoji = "🟢" if pct >= 80 else "🟡" if pct >= 60 else "🔴"
    lines.append(f"{emoji} **{pct}%** ({eval_result['total_earned']:.1f} / {eval_result['total_max']:.1f})")
    lines.append("")

    # Per-category breakdown
    lines.append("## Per-Category Scores")
    lines.append("")
    lines.append("| Category | Earned | Max | Score |")
    lines.append("|----------|--------|-----|-------|")
    for cat, scores in sorted(eval_result["category_scores"].items()):
        earned = scores["earned"]
        mx = scores["max"]
        cat_pct = round(earned / mx * 100, 1) if mx > 0 else 0
        lines.append(f"| {cat} | {earned:.1f} | {mx:.1f} | {cat_pct}% |")
    lines.append("")

    # Detailed results per question
    lines.append("## Detailed Results")
    lines.append("")
    lines.append("| ID | Category | Query (truncated) | Key Scores | Top Hit | Graph Hits | Details |")
    lines.append("|----|----------|-------------------|------------|---------|------------|---------|")

    for r in eval_result["results"]:
        if "error" in r:
            lines.append(f"| {r['id']} | {r['category']} | {r['query'][:30]}... | ERROR | - | - | {r['error']} |")
            continue

        # Summarize key scores
        score_parts = []
        for cat, sr in r["scores"].items():
            cat_short = cat[:4].upper()
            earned = sr["score"]
            mx = sr["max_score"]
            if mx > 0:
                score_parts.append(f"{cat_short}:{earned:.0f}/{mx:.0f}")
        score_str = ", ".join(score_parts) if score_parts else "-"

        details = ""
        for cat, sr in r["scores"].items():
            if sr["details"]:
                details += f"[{cat[:4]}] {sr['details']}; "

        top_hit = f"{r.get('top_hit_score', 0):.3f}"
        graph_hits = r.get("graph_suggestions_count", 0)

        lines.append(f"| {r['id']} | {r['category']} | {r['query'][:30]}... | {score_str} | {top_hit} | {graph_hits} | {details[:80]}... |")

    lines.append("")

    # Key findings
    lines.append("## Key Findings")
    lines.append("")

    # Component inference accuracy
    ci = eval_result["category_scores"].get("component_inference", {"earned": 0, "max": 1})
    ci_pct = round(ci["earned"] / ci["max"] * 100, 1) if ci["max"] > 0 else 0
    lines.append(f"- **Component Inference**: {ci_pct}% accuracy")

    # Graph matching recall
    gm = eval_result["category_scores"].get("graph_matching", {"earned": 0, "max": 1})
    gm_pct = round(gm["earned"] / gm["max"] * 100, 1) if gm["max"] > 0 else 0
    lines.append(f"- **Graph Matching**: {gm_pct}% recall (expected entries found)")

    # RAG retrieval
    rr = eval_result["category_scores"].get("rag_recall", {"earned": 0, "max": 1})
    rr_pct = round(rr["earned"] / rr["max"] * 100, 1) if rr["max"] > 0 else 0
    lines.append(f"- **RAG Recall**: {rr_pct}% (top hits above threshold)")

    # Advice consistency
    ac = eval_result["category_scores"].get("advice_consistency", {"earned": 0, "max": 1})
    ac_pct = round(ac["earned"] / ac["max"] * 100, 1) if ac["max"] > 0 else 0
    lines.append(f"- **Advice Consistency**: {ac_pct}% keywords found")

    # Safety compliance
    sc = eval_result["category_scores"].get("safety_compliance", {"earned": 0, "max": 1})
    sc_pct = round(sc["earned"] / sc["max"] * 100, 1) if sc["max"] > 0 else 0
    lines.append(f"- **Safety Compliance**: {sc_pct}%")

    # Risk assessment
    ra = eval_result["category_scores"].get("risk_assessment", {"earned": 0, "max": 1})
    ra_pct = round(ra["earned"] / ra["max"] * 100, 1) if ra["max"] > 0 else 0
    lines.append(f"- **Risk Assessment**: {ra_pct}% accuracy")

    lines.append("")

    # Search quality metrics
    lines.append("## Search Quality Metrics")
    lines.append("")
    hits_scores = []
    for r in eval_result["results"]:
        if "error" not in r and r.get("top_hit_score"):
            hits_scores.append(r["top_hit_score"])

    if hits_scores:
        avg_score = sum(hits_scores) / len(hits_scores)
        min_score = min(hits_scores)
        max_score = max(hits_scores)
        lines.append(f"- **Average top-hit score**: {avg_score:.3f}")
        lines.append(f"- **Min top-hit score**: {min_score:.3f}")
        lines.append(f"- **Max top-hit score**: {max_score:.3f}")
        lines.append(f"- **Queries with score >= 0.6**: {sum(1 for s in hits_scores if s >= 0.6)}/{len(hits_scores)}")
        lines.append(f"- **Queries with score >= 0.5**: {sum(1 for s in hits_scores if s >= 0.5)}/{len(hits_scores)}")
    lines.append("")

    # Recommendations
    lines.append("## Recommendations")
    lines.append("")

    if gm_pct < 80:
        lines.append("- **Graph coverage gap**: Some expected fault graph entries not matched. Consider adding more specific fault_mode/symptom mappings to the graph.")
    if ci_pct < 80:
        lines.append("- **Component inference gap**: Some queries not mapped to the correct component. Review normalize.rs inference rules.")
    if sc_pct < 80:
        lines.append("- **Safety compliance gap**: Some safety-trigger questions missing required safety notes or human confirmation. Review advice.rs safety keyword detection.")
    if rr_pct < 80:
        lines.append("- **RAG recall gap**: Some queries returning low-relevance hits. Consider adding more domain-specific documents or adjusting search weights (0.65/0.25/0.10).")
    if ac_pct < 80:
        lines.append("- **Advice consistency gap**: Some expected keywords missing from advice. Review advice generation rules and graph entry maintenance_actions.")

    if pct >= 80:
        lines.append("- Overall performance is good. Focus on edge cases and low-scoring categories for improvement.")
    elif pct >= 60:
        lines.append("- Moderate performance. Priority: fix safety compliance gaps, then improve graph coverage and component inference.")
    else:
        lines.append("- Significant gaps detected. Priority order: 1) Safety compliance, 2) Graph coverage, 3) Component inference, 4) RAG recall.")

    lines.append("")
    lines.append("---")
    lines.append(f"*Generated by beifeng/evals/run_benchmark.py at {time.strftime('%Y-%m-%d %H:%M:%S')}*")

    return "\n".join(lines)


def main():
    import argparse
    parser = argparse.ArgumentParser(description="BeiFeng Benchmark Evaluation Runner")
    parser.add_argument("--url", default=DEFAULT_URL, help=f"RAG service URL (default: {DEFAULT_URL})")
    parser.add_argument("--output", default=None, help="Output report path (default: beifeng/evals/benchmark_report_<date>.md)")
    parser.add_argument("--json-output", default=None, help="Output raw JSON results path")
    parser.add_argument("--report", action="store_true", help="Enable report_generation CLI evaluation")
    args = parser.parse_args()

    # Load benchmark
    if not BENCHMARK_PATH.exists():
        print(f"ERROR: Benchmark file not found: {BENCHMARK_PATH}")
        sys.exit(1)

    with open(BENCHMARK_PATH, "r", encoding="utf-8") as f:
        benchmark = json.load(f)

    print(f"Loaded {len(benchmark.get('questions', []))} benchmark questions")
    print(f"Dimensions: {', '.join(benchmark.get('dimensions', []))}")

    # Check health
    print(f"Checking RAG service at {args.url}...")
    if not check_health(args.url):
        print(f"ERROR: RAG service not reachable at {args.url}")
        print("Start it with: CLAW_RAG_MOCK_PROVIDERS=1 cargo run -p claw-rag-service -- serve --db ./beifeng/data/wind.sqlite --graph ./beifeng/knowledge/knowledge_graph/wind_fault_graph.json")
        sys.exit(1)

    print("RAG service is healthy. Starting evaluation...\n")

    # Run evaluation
    eval_result = run_evaluation(args.url, benchmark, enable_report=args.report)

    # Format report
    report = format_report(eval_result, benchmark)

    # Save report
    output_path = args.output
    if output_path is None:
        output_path = str(Path(__file__).resolve().parent / f"benchmark_report_{time.strftime('%Y%m%d_%H%M%S')}.md")

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(report)
    print(f"\nReport saved to: {output_path}")

    # Save JSON if requested
    if args.json_output:
        with open(args.json_output, "w", encoding="utf-8") as f:
            json.dump(eval_result, f, ensure_ascii=False, indent=2)
        print(f"JSON results saved to: {args.json_output}")

    # Print summary
    print(f"\n{'='*60}")
    print(f"OVERALL SCORE: {eval_result['overall_pct']}% ({eval_result['total_earned']:.1f}/{eval_result['total_max']:.1f})")
    print(f"{'='*60}")
    for cat, scores in sorted(eval_result["category_scores"].items()):
        cat_pct = round(scores["earned"] / scores["max"] * 100, 1) if scores["max"] > 0 else 0
        print(f"  {cat}: {cat_pct}% ({scores['earned']:.1f}/{scores['max']:.1f})")


if __name__ == "__main__":
    main()
