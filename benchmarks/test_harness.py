import unittest
from run import canonical
class ParityTests(unittest.TestCase):
 def test_formatting(self):self.assertEqual(canonical('<p>Hello   world</p>'),canonical('<p>Hello world</p>'))
 def test_semantics(self):self.assertNotEqual(canonical('<p><strong>x</strong></p>'),canonical('<p>x</p>'))
 def test_links(self):self.assertNotEqual(canonical('<a href="a">x</a>'),canonical('<a href="b">x</a>'))
 def test_code_whitespace(self):self.assertNotEqual(canonical('<pre> x</pre>'),canonical('<pre>x</pre>'))
if __name__=='__main__':unittest.main()
