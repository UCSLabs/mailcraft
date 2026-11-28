// SPDX-FileCopyrightText: MailCraft contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
package work.mailcraft.android

import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class LanguagesTest {
    @Test
    fun theCoreSpeaksWhatTheLocaleAsksForOrEnglish() {
        assertEquals("fr", coreLanguage("fr"))
        assertEquals("de", coreLanguage("DE"))
        assertEquals("nl", coreLanguage("nl-NL"))
        assertEquals("ro", coreLanguage("ro_RO"))
        assertEquals("hu", coreLanguage("hu-HU"))
        assertEquals("pl", coreLanguage("pl-PL"))
        assertEquals("sl", coreLanguage("sl-SI"))
        assertEquals("cs", coreLanguage("cs-CZ"))
        assertEquals("ka", coreLanguage("ka-GE"))
        assertEquals("pt-BR", coreLanguage("pt"))
        assertEquals("pt-BR", coreLanguage("pt-BR"))
        assertEquals("pt-PT", coreLanguage("pt-PT"))
        assertEquals("pt-PT", coreLanguage("pt_PT"))
        assertEquals("pt-PT", coreLanguage("pt-PT-u-ca-gregory"))
        assertEquals("pt-PT", coreLanguage("pt-Latn-PT"))
        assertEquals("pt-BR", coreLanguage("pt-BR-x-pt"))
        assertEquals("en", coreLanguage(""))
        assertEquals("en", coreLanguage(null))
        assertEquals("en", coreLanguage("\u0000￿"))
    }

    @Test
    fun theChoiceIsTheFirstLanguageThatIsOffered() {
        assertEquals("", chosenTag(emptyList()))
        assertEquals("fr", chosenTag(listOf("fr-CA")))
        assertEquals("pt-BR", chosenTag(listOf("pt-BR", "es-MX", "fr")))
        assertEquals("pt-PT", chosenTag(listOf("pt-PT", "fr")))
        assertEquals("pt-PT", chosenTag(listOf("pt-PT-u-ca-gregory", "fr")))
        assertEquals("pt-PT", chosenTag(listOf("pt-Latn-PT", "fr")))
        assertEquals("pt-BR", chosenTag(listOf("pt-BR-x-pt", "fr")))
        assertEquals("pt-BR", chosenTag(listOf("pt", "fr")))
        assertEquals("es", chosenTag(listOf("ja", "es-MX", "fr")))
        assertEquals("", chosenTag(listOf("ja")))
        assertEquals("", chosenTag(listOf("")))
    }

    @Test
    fun theListOfLanguagesIsTheOneTheSystemIsTold() {
        // The test runs in the app's folder.
        val xml = File("src/main/res/xml/locales_config.xml").readText()
        val told = Regex("""<locale android:name="([^"]+)"""").findAll(xml).map { it.groupValues[1] }.toList()
        assertEquals(LANGUAGES.map { it.tag }, told)
        assertEquals(told.size, told.toSet().size)
    }

    @Test
    fun everyLanguageHasACatalogueOrIsEnglish() {
        // crates/i18n/po/<code>.po, from the app's folder. The two Portuguese catalogues are
        // pt_BR.po (tag `pt-BR`) and pt_PT.po (tag `pt-PT`).
        val po = File("../../../crates/i18n/po")
        LANGUAGES.filter { it.tag != "en" }.forEach { lang ->
            val file = when (lang.tag) {
                "pt-BR" -> "pt_BR.po"
                "pt-PT" -> "pt_PT.po"
                else -> "${lang.tag}.po"
            }
            assertTrue(lang.tag, File(po, file).isFile)
        }
    }

    @Test
    fun namesAreTheLanguagesOwn() {
        assertEquals("Deutsch", LANGUAGES.first { it.tag == "de" }.nativeName)
        assertTrue(LANGUAGES.all { it.nativeName.isNotBlank() })
    }
}
