// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use rand::seq::IndexedRandom;

pub fn generate_localized_twist(locale: &str) -> String {
    let mut rng = rand::rng();

    match locale {
        "ru" => {
            let catalysts = [
                "Тайный советник ордена",
                "Преданный оруженосец",
                "Беглый картограф империи",
                "Торговец забытыми реликвиями",
                "Странник из зеркального измерения",
                "Истинный наследник престола",
                "Архивариус запретной крипты",
                "Опальный алхимик",
            ];
            let twists = [
                "невольно взламывает защитный барьер цитадели",
                "находит доказательства того, что война окончилась век назад",
                "теряет способность лгать перед священным пламенем",
                "обнаруживает в древнем фолианте портрет собственного предка",
                "похищает ключ от крипты забвения ради спасения врага",
                "снимает маску, обнажая метку павшего демиурга",
                "осознаёт, что артефакт питается воспоминаниями его близких",
            ];
            let climaxes = [
                "и теперь обратный отсчёт мироздания необратим.",
                "что ставит под удар всю исследовательскую экспедицию.",
                "пока на северной границе собираются тени прошлого.",
                "хотя цена этой правды уже оплачена чужой жертвой.",
                "вынуждая враждующие стороны заключить шаткое перемирие.",
            ];
            let c = catalysts.choose(&mut rng).unwrap_or(&"Герой");
            let t = twists.choose(&mut rng).unwrap_or(&"совершает роковой шаг");
            let cl = climaxes.choose(&mut rng).unwrap_or(&"навсегда.");
            format!("{} {}, {}", c, t, cl)
        }
        "de" => {
            let catalysts = [
                "Ein geheimer Berater des Ordens",
                "Der treue Knappe",
                "Ein flüchtiger Kartograf des Reiches",
                "Der Händler vergessener Relikte",
                "Ein Wanderer aus der Spiegeldimension",
                "Der wahre Thronerbe",
                "Der Archivar der verbotenen Krypta",
                "Ein verstoßener Alchemist",
            ];
            let twists = [
                "bricht unabsichtlich die Schutzbarriere der Zitadelle",
                "entdeckt Beweise dafür, dass der Krieg vor einem Jahrhundert endete",
                "verliert vor der heiligen Flamme die Fähigkeit zu lügen",
                "findet im alten Folianten ein Porträt des eigenen Vorfahren",
                "stiehlt den Schlüssel zur Krypta des Vergessens, um einen Feind zu retten",
                "nimmt die Maske ab und enthüllt das Mal eines gefallenen Demiurgen",
                "erkennt, dass das Artefakt von den Erinnerungen seiner Liebsten zehrt",
            ];
            let climaxes = [
                "und nun ist der Countdown der Welt unumkehrbar.",
                "was die gesamte Expedition in höchste Gefahr bringt.",
                "während sich an der Nordgrenze die Schatten der Vergangenheit sammeln.",
                "obwohl der Preis dieser Wahrheit längst bezahlt wurde.",
                "und zwingt die verfeindeten Seiten zu einem brüchigen Waffenstillstand.",
            ];
            let c = catalysts.choose(&mut rng).unwrap_or(&"Der Held");
            let t = twists.choose(&mut rng).unwrap_or(&"trifft eine schicksalhafte Entscheidung");
            let cl = climaxes.choose(&mut rng).unwrap_or(&"für immer.");
            format!("{} {}, {}", c, t, cl)
        }
        "es" => {
            let catalysts = [
                "El consejero secreto de la orden",
                "El escudero devoto",
                "Un cartógrafo fugitivo del imperio",
                "El mercader de reliquias olvidadas",
                "Un viajero de la dimensión espejo",
                "El verdadero heredero al trono",
                "El archivista de la cripta prohibida",
                "Un alquimista caído en desgracia",
            ];
            let twists = [
                "rompe involuntariamente la barrera protectora de la ciudadela",
                "halla pruebas de que la guerra terminó hace un siglo",
                "pierde la capacidad de mentir ante la llama sagrada",
                "descubre en un antiguo códice el retrato de su propio ancestro",
                "roba la llave de la cripta del olvido para salvar a un enemigo",
                "se quita la máscara revelando la marca de un demiurgo caído",
                "comprende que el artefacto se alimenta de los recuerdos de sus seres queridos",
            ];
            let climaxes = [
                "y ahora la cuenta atrás del mundo es irreversible.",
                "poniendo en peligro a toda la expedición.",
                "mientras las sombras del pasado se reúnen en la frontera norte.",
                "aunque el precio de esta verdad ya haya sido pagado.",
                "obligando a las facciones rivales a firmar una tregua frágil.",
            ];
            let c = catalysts.choose(&mut rng).unwrap_or(&"El protagonista");
            let t = twists.choose(&mut rng).unwrap_or(&"da un paso irreversible");
            let cl = climaxes.choose(&mut rng).unwrap_or(&"para siempre.");
            format!("{} {}, {}", c, t, cl)
        }
        _ => {
            let catalysts = [
                "A secret councilor of the order",
                "The devoted squire",
                "A fugitive imperial cartographer",
                "The merchant of forgotten relics",
                "A wanderer from the mirror realm",
                "The rightful heir to the throne",
                "The archivist of the forbidden crypt",
                "An exiled alchemist",
            ];
            let twists = [
                "inadvertently shatters the citadel's protective ward",
                "uncovers proof that the war ended a century ago",
                "loses the ability to lie before the sacred flame",
                "discovers a portrait of their own ancestor inside an ancient tome",
                "steals the key to the crypt of oblivion to rescue an enemy",
                "removes the mask, revealing the mark of a fallen demiurge",
                "realizes the artifact feeds on the memories of their loved ones",
            ];
            let climaxes = [
                "making the cosmic countdown irreversible.",
                "putting the entire expedition at mortal risk.",
                "while shadows of the past gather on the northern border.",
                "though the price of that truth was already paid in another life.",
                "forcing rival factions into a fragile truce.",
            ];
            let c = catalysts.choose(&mut rng).unwrap_or(&"The hero");
            let t = twists.choose(&mut rng).unwrap_or(&"makes a fateful choice");
            let cl = climaxes.choose(&mut rng).unwrap_or(&"forever.");
            format!("{} {}, {}", c, t, cl)
        }
    }
}