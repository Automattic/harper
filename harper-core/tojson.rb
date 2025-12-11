# Converter from Hunspell affix format to HArper's json

affix_template = '		"%{letter}": {
			"#": "-",
			"kind": "%{sufpref}",
			"cross_product": true,
			"replacements": [
%{replacements}			],
			"target": [],
			"base_metadata": {},
			"rename_ok": true
		},
'

replacement_template = '				{
					"remove": "%{remove}",
					"add": "%{add}",
					"condition": "%{condition}"
				},
'

remap_alphabet = "анісырлеокятдувзмпцўбгґчьшхйэжюёАНПЯІСКТВМДЗГҐУЛБфШЁХРЧЦЮЖОФЎЭЕЫЙЬ".split(//)
remap_tbl = {}

json_out = ""
aff_out = ""

curjson = ""
json_entries = ""

File.read(ARGV[0]).lines.each do |l|
  if l =~ /(SFX|PFX)\s+(\S+)\s+((\S+)\s+(\S+)(\s+(\S+))?)/
    sufpref = $1
    letter = $2
    other = $3
    x1 = $4
    x2 = $5
    x3 = $7
    x1 = "" if x1 == "0"
    x2 = "" if x2 == "0"
    newletter = ""
    unless remap_tbl.has_key?(letter)
      newletter = remap_alphabet.shift
      remap_tbl[letter] = newletter
      json_out = json_out + curjson.gsub("%{replacements}", json_entries.gsub(/,(\s*)\Z/, "\\1"))
      curjson = affix_template.gsub("%{letter}", newletter)
                                 .gsub("%{sufpref}", ((sufpref == "SFX") ? "suffix" : "prefix"))
      json_entries = ""
    else
      newletter = remap_tbl[letter]
      json_entry = affix_template.gsub("%{letter}", newletter)
                                 .gsub("%{sufpref}", ((sufpref == "SFX") ? "suffix" : "prefix"))
      json_entries = json_entries +
        replacement_template.gsub("%{remove}", x1)
                            .gsub("%{add}", x2)
                            .gsub("%{condition}", x3)
    end
    aff_out = aff_out + "#{sufpref} #{newletter} #{other}\n"
  end
end

json_out = json_out + curjson.gsub("%{replacements}", json_entries.gsub(/,(\s*)\Z/, "\\1"))

File.write("_affout.aff", aff_out)
File.write("_jsonout.json", json_out)
