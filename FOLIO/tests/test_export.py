import json
import pathlib
import shutil
import tempfile
import unittest

from folio.export import export_rows


class TestExportRows(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _out(self) -> pathlib.Path:
        return self.tmp / "generated"

    def test_exports_entails_refutes_and_metadata(self):
        row = {
            "story-id": "story-1",
            "example-id": "ex/1",
            "premises-FOL": ["∀x (Dog(x) → Animal(x))", "Dog(fido)"],
            "conclusion-FOL": "Animal(fido)",
            "label": "True",
        }
        out = self._out()
        exported = export_rows([row], "validation", out)

        entails = out / "validation__ex_1__entails.p"
        refutes = out / "validation__ex_1__refutes.p"
        self.assertEqual(len(exported), 1)
        self.assertTrue(entails.exists())
        self.assertTrue(refutes.exists())
        self.assertIn("fof(premise_1,axiom,(! [X] : (dog(X) => animal(X)))).", entails.read_text())
        self.assertIn("fof(conclusion,conjecture,animal(fido)).", entails.read_text())
        self.assertIn("fof(conclusion_negated,conjecture,~(animal(fido))).", refutes.read_text())

        records = [json.loads(line) for line in (out / "metadata.jsonl").read_text().splitlines()]
        self.assertEqual(records[0]["example_id"], "ex/1")
        self.assertEqual(records[0]["entails_file"], str(entails))

    def test_unsupported_conclusion_is_recorded(self):
        row = {
            "example-id": "eq",
            "premises-FOL": ["Dog(fido)"],
            "conclusion-FOL": "fido = fido",
            "label": "True",
        }
        out = self._out()
        exported = export_rows([row], "validation", out)

        self.assertEqual(exported, [])
        records = [json.loads(line) for line in (out / "unsupported.jsonl").read_text().splitlines()]
        self.assertEqual(records[0]["example_id"], "eq")
        self.assertEqual(records[0]["field"], "conclusion-FOL")
        self.assertIn("equality", records[0]["reason"])

    def test_unsupported_premise_skips_whole_example(self):
        row = {
            "example-id": "bad-premise",
            "premises-FOL": ["a = b", "Dog(fido)"],
            "conclusion-FOL": "Dog(fido)",
            "label": "True",
        }
        out = self._out()
        exported = export_rows([row], "validation", out)
        self.assertEqual(exported, [])

    def test_multiple_rows_exports_all_supported(self):
        good = {
            "example-id": "good",
            "premises-FOL": ["Dog(fido)"],
            "conclusion-FOL": "Dog(fido)",
            "label": "True",
        }
        bad = {
            "example-id": "bad",
            "premises-FOL": ["a = b"],
            "conclusion-FOL": "Dog(fido)",
            "label": "True",
        }
        out = self._out()
        exported = export_rows([good, bad, good], "validation", out)
        self.assertEqual(len(exported), 2)

    def test_example_id_sanitisation(self):
        row = {
            "example-id": "ex/with spaces",
            "premises-FOL": ["Dog(fido)"],
            "conclusion-FOL": "Dog(fido)",
            "label": "True",
        }
        out = self._out()
        exported = export_rows([row], "validation", out)
        self.assertEqual(len(exported), 1)
        self.assertTrue(exported[0].entails_file.name.startswith("validation__ex_with_spaces__"))


if __name__ == "__main__":
    unittest.main()
