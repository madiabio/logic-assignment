import csv
import io
import pathlib
import shutil
import tempfile
import unittest
from contextlib import redirect_stdout
from unittest.mock import patch

from folio.db import ResultDetail, RunConstraints
from folio.evaluate import predict, write_evaluation
from folio.export import ExportedExample


def _example(example_id: str, label: str, out_dir: pathlib.Path) -> ExportedExample:
    return ExportedExample(
        split="validation",
        story_id="",
        example_id=example_id,
        label=label,
        entails_file=out_dir / f"validation__{example_id}__entails.p",
        refutes_file=out_dir / f"validation__{example_id}__refutes.p",
    )


def _detail(status: str, elapsed_ms: int = 100, unknown_reason: str | None = None) -> ResultDetail:
    return ResultDetail(status=status, elapsed_ms=elapsed_ms, unknown_reason=unknown_reason)


class TestPredict(unittest.TestCase):
    def test_true(self):
        self.assertEqual(predict("provable", "not_provable"), "true")

    def test_false(self):
        self.assertEqual(predict("not_provable", "provable"), "false")

    def test_unknown(self):
        self.assertEqual(predict("not_provable", "not_provable"), "unknown")

    def test_inconsistent(self):
        self.assertEqual(predict("provable", "provable"), "inconsistent")

    def test_undetermined_on_timeout_entails(self):
        self.assertEqual(predict("timeout", "provable"), "undetermined")

    def test_undetermined_on_timeout_refutes(self):
        self.assertEqual(predict("provable", "timeout"), "undetermined")

    def test_undetermined_on_cancelled(self):
        self.assertEqual(predict("cancelled", "not_provable"), "undetermined")

    def test_undetermined_on_missing(self):
        self.assertEqual(predict("missing", "missing"), "undetermined")

    def test_undetermined_on_error(self):
        self.assertEqual(predict("error", "not_provable"), "undetermined")

    def test_undetermined_on_not_implemented(self):
        self.assertEqual(predict("not_implemented", "provable"), "undetermined")


class TestWriteEvaluation(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _run(self, exported, details, constraints=None):
        buf = io.StringIO()
        # Redirect TSV output to tmp dir
        with patch("folio.evaluate.SUMMARY_TSV", self.tmp / "summary.tsv"), \
             patch("folio.evaluate.PREDICTIONS_TSV", self.tmp / "predictions.tsv"), \
             redirect_stdout(buf):
            write_evaluation(exported, details, constraints)
        return buf.getvalue()

    def test_prints_constraint_header(self):
        ex = _example("42", "true", self.tmp)
        details = {
            "validation__42__entails": _detail("provable"),
            "validation__42__refutes": _detail("not_provable"),
        }
        constraints = RunConstraints(
            label="test-run",
            engine="priority-id",
            timeout_ms=1000,
            max_depth=128,
            max_steps=50000,
            max_fresh_terms_per_quantifier=1,
            problem_class="mixed",
        )
        output = self._run([ex], details, constraints)
        self.assertIn("test-run", output)
        self.assertIn("engine=priority-id", output)
        self.assertIn("timeout=1000ms", output)

    def test_prints_per_example_row(self):
        ex = _example("99", "true", self.tmp)
        details = {
            "validation__99__entails": _detail("provable"),
            "validation__99__refutes": _detail("not_provable"),
        }
        output = self._run([ex], details)
        self.assertIn("99", output)
        self.assertIn("true", output)

    def test_writes_predictions_tsv(self):
        ex = _example("1", "true", self.tmp)
        details = {
            "validation__1__entails": _detail("provable"),
            "validation__1__refutes": _detail("not_provable"),
        }
        self._run([ex], details)
        tsv = self.tmp / "predictions.tsv"
        self.assertTrue(tsv.exists())
        rows = list(csv.DictReader(tsv.open(), delimiter="\t"))
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["example_id"], "1")
        self.assertEqual(rows[0]["prediction"], "true")
        self.assertEqual(rows[0]["gold"], "true")

    def test_writes_summary_tsv(self):
        examples = [_example("a", "true", self.tmp), _example("b", "false", self.tmp)]
        details = {
            "validation__a__entails": _detail("provable"),
            "validation__a__refutes": _detail("not_provable"),
            "validation__b__entails": _detail("not_provable"),
            "validation__b__refutes": _detail("provable"),
        }
        self._run(examples, details)
        tsv = self.tmp / "summary.tsv"
        rows = list(csv.reader(tsv.open(), delimiter="\t"))
        # header + two data rows
        self.assertEqual(rows[0], ["prediction", "gold", "count"])
        data = {(r[0], r[1]): int(r[2]) for r in rows[1:]}
        self.assertEqual(data[("true", "true")], 1)
        self.assertEqual(data[("false", "false")], 1)


if __name__ == "__main__":
    unittest.main()
