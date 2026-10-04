import copy
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import validate_four_world_catalogs as audit  # noqa: E402


class FourWorldNarrativeCatalogAuditTests(unittest.TestCase):
    def setUp(self):
        self.catalogs = {world: json.loads(path.read_text(encoding="utf-8"))
                         for world, path in audit.CATALOGS.items()}
        self.scenes = audit._load_scene_context()
        self.progression = audit._load_progression()
        self.source_lines = audit.DEFAULT_SOURCE.read_text(encoding="utf-8").splitlines()

    def check(self, catalogs=None, source_lines=None, source_sha=None):
        return audit.audit_catalogs(
            catalogs if catalogs is not None else self.catalogs,
            self.scenes,
            self.progression,
            source_lines if source_lines is not None else self.source_lines,
            source_sha if source_sha is not None else audit.EXPECTED_SOURCE_SHA256,
        )

    def test_all_four_existing_validators_and_cross_audit_pass(self):
        result = audit.validate_files()
        self.assertEqual(result["result"], "PASS")
        self.assertEqual(result["worldCount"], 4)
        self.assertEqual(result["sceneCount"], 30)
        self.assertEqual(result["globalNarrativeIdCount"], 55)
        self.assertEqual(result["totals"]["confirmedBodyCount"], 19)
        self.assertEqual(result["totals"]["pendingBodyCount"], 36)
        self.assertEqual(result["totals"]["pendingSceneHookCount"], 7)
        self.assertEqual({k: v["result"] for k, v in result["existingValidators"].items()},
                         {world: "PASS" for world in audit.EXPECTED_SCENES})

    def test_rejects_global_id_collision(self):
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["return_station"]["entries"][0]["id"] = "gh_sys_arrival"
        with self.assertRaisesRegex(audit.AuditError, "not globally unique"):
            self.check(catalogs=catalogs)

    def test_rejects_missing_scene_membership_or_wrong_world_identity(self):
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["grey_hive"]["scenes"].remove("gh_exit")
        with self.assertRaisesRegex(audit.AuditError, "scene membership is incorrect"):
            self.check(catalogs=catalogs)
        scenes = copy.deepcopy(self.scenes)
        scenes["return_station"]["rs_core_room"]["worldId"] = "grey_hive"
        with self.assertRaisesRegex(audit.AuditError, "mismatched ID/world identity"):
            audit.audit_catalogs(self.catalogs, scenes, self.progression, self.source_lines,
                                 audit.EXPECTED_SOURCE_SHA256)

    def test_confirmed_copy_and_line_must_match_authoritative_table(self):
        catalogs = copy.deepcopy(self.catalogs)
        cw_end = next(row for row in catalogs["clockworks"]["entries"] if row["id"] == "cw_cy_end_01")
        cw_end["body"] = "门后的世界很辽阔。"
        with self.assertRaisesRegex(audit.AuditError, "not authoritative"):
            self.check(catalogs=catalogs)
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["grey_hive"]["entries"][0]["sourceLine"] = 611
        with self.assertRaisesRegex(audit.AuditError, "outside grey_hive design table|not authoritative"):
            self.check(catalogs=catalogs)

    def test_confirmed_text_must_be_present_on_its_source_line(self):
        lines = list(self.source_lines)
        lines[694] = lines[694].replace("生产线无人值守，但压力循环仍在运行。", "替换的错误文本")
        with self.assertRaisesRegex(audit.AuditError, "absent from its authoritative source line"):
            self.check(source_lines=lines)

    def test_source_sha_is_pinned_and_all_catalog_sha_metadata_agrees(self):
        with self.assertRaisesRegex(audit.AuditError, "SHA-256 does not match"):
            self.check(source_sha="0" * 64)
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["mist_harbor"]["designSourceSha256"] = "0" * 64
        with self.assertRaisesRegex(audit.AuditError, "source SHA-256 differs"):
            self.check(catalogs=catalogs)
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["grey_hive"].pop("sourceLineRange")
        with self.assertRaisesRegex(audit.AuditError, "sourceLineRange"):
            self.check(catalogs=catalogs)

    def test_pending_entries_cannot_gain_unsourced_prose(self):
        catalogs = copy.deepcopy(self.catalogs)
        pending = next(row for row in catalogs["mist_harbor"]["entries"] if row["bodyStatus"] == "pending")
        pending["body"] = "灯塔已经恢复。"
        with self.assertRaisesRegex(audit.AuditError, "pending body must be null"):
            self.check(catalogs=catalogs)

    def test_mist_harbor_events_remain_compiled_only_and_capability_pending(self):
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["mist_harbor"]["progressionEvents"][0]["nativePlayableClaimed"] = True
        with self.assertRaisesRegex(audit.AuditError, "must not claim native playability"):
            self.check(catalogs=catalogs)
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["mist_harbor"]["capabilities"][0]["status"] = "implemented"
        with self.assertRaisesRegex(audit.AuditError, "must remain unimplemented"):
            self.check(catalogs=catalogs)

    def test_clockworks_events_remain_pending_native_implementation(self):
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["clockworks"]["progressionEvents"][0]["status"] = "complete"
        with self.assertRaisesRegex(audit.AuditError, "must remain pending native implementation"):
            self.check(catalogs=catalogs)

    def test_return_station_pending_states_and_confirmed_hook_boundary_are_preserved(self):
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["return_station"]["implementationClaims"]["mistHarborReturnAvailable"] = True
        with self.assertRaisesRegex(audit.AuditError, "overstates pending return"):
            self.check(catalogs=catalogs)
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["return_station"]["entries"][0]["triggerStatus"] = "bound"
        with self.assertRaisesRegex(audit.AuditError, "must remain pending its scene hook"):
            self.check(catalogs=catalogs)

    def test_grey_hive_gate_b_progression_and_choice_boundary_are_preserved(self):
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["grey_hive"]["progressionConstraints"]["baizhiChoiceIsGateBKey"] = True
        with self.assertRaisesRegex(audit.AuditError, "progression constraints are inconsistent"):
            self.check(catalogs=catalogs)

    def test_design_source_metadata_and_world_groups_are_complete(self):
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["clockworks"]["designSource"] = "other-report.md"
        with self.assertRaisesRegex(audit.AuditError, "canonical Markdown design source"):
            self.check(catalogs=catalogs)
        with self.assertRaisesRegex(audit.AuditError, "four world catalogs and scene groups"):
            audit.audit_catalogs(self.catalogs, {"grey_hive": {}}, self.progression, self.source_lines,
                                 audit.EXPECTED_SOURCE_SHA256)

    def test_rejects_incorrect_section_line_range_even_when_rows_are_valid(self):
        catalogs = copy.deepcopy(self.catalogs)
        catalogs["grey_hive"]["sourceLineRange"] = "612-622"
        with self.assertRaisesRegex(audit.AuditError, "sourceLineRange does not match"):
            self.check(catalogs=catalogs)


if __name__ == "__main__":
    unittest.main()
