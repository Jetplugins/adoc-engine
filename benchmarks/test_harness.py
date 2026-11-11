import unittest
from run import canonical, comparison_tokens
class ParityTests(unittest.TestCase):
 def test_formatting(self):self.assertEqual(canonical('<p>Hello   world</p>'),canonical('<p>Hello world</p>'))
 def test_semantics(self):self.assertNotEqual(canonical('<p><strong>x</strong></p>'),canonical('<p>x</p>'))
 def test_inline_word_spacing(self):self.assertNotEqual(canonical('<p>A <strong>B</strong> C</p>'),canonical('<p>A<strong>B</strong>C</p>'))
 def test_between_inline_nodes(self):self.assertNotEqual(canonical('<p><strong>A</strong> <em>B</em></p>'),canonical('<p><strong>A</strong><em>B</em></p>'))
 def test_nonbreaking_space(self):self.assertNotEqual(canonical('<p>A&nbsp;B</p>'),canonical('<p>A B</p>'))
 def test_links(self):self.assertNotEqual(canonical('<a href="a">x</a>'),canonical('<a href="b">x</a>'))
 def test_code_whitespace(self):self.assertNotEqual(canonical('<pre> x</pre>'),canonical('<pre>x</pre>'))
 def test_js_column_width_equivalence(self):
  self.assertEqual(comparison_tokens('js', '<col width="50%">'), canonical('<col style="width: 50%;">'))
 def test_rust_column_width_remains_strict(self):
  self.assertNotEqual(comparison_tokens('rust', '<col width="50%">'), canonical('<col style="width: 50%;">'))
 def test_column_width_differences_remain_visible(self):
  self.assertNotEqual(comparison_tokens('js', '<col width="40%">'), canonical('<col style="width: 50%;">'))
 def test_column_style_is_not_overridden(self):
  self.assertNotEqual(comparison_tokens('js', '<col width="50%" style="width: 20%;">'), canonical('<col style="width: 50%;">'))
 def test_other_width_attributes_remain_strict(self):
  self.assertNotEqual(comparison_tokens('js', '<img width="50%">'), canonical('<img style="width: 50%;">'))
if __name__=='__main__':unittest.main()
