package newish

import (
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"
	"unicode"

	"github.com/PuerkitoBio/goquery"

	"time"

	"github.com/gocolly/colly"
)

type parsingStage uint

const (
	parsingStageundefined parsingStage = iota
	parsingStagecharacteristics
	parsingStagepreRequisits
	parsingStageexams
)

type examsStage uint

const (
	examsStageUndefined examsStage = iota
)

type examsType uint

const (
	examsTypeUndefined examsType = iota
	examsTypeMandatory
	examsTypeGroups
)

type newExamsInfo struct {
	mandatoryExams     []exam
	optionalExams      []exam
	optionalExamGroups [][]exam
}

type examsInfo struct {
	description string
	etype       examsType
	exams       []exam
	examGroups  [][]exam
}

type exam struct {
	code uint8
	name string
}

type cnaef struct {
	code uint8
	name string
}

type duration struct {
	code uint8
	name string // semestres / anos
}

type courseInfo struct {
	courseName      string //! not needed
	institutionName string //! not needed
	courseCode      string // foreign key
	institutionCode string // foreign key
	degree          string // foreign key
	ects            uint16
	cnaef           string // split into code + name
	duration        string // duration struct
	ctype           string // foreign key
	contest         string // foreign key
	examsInfo       examsInfo
}

func main() {

	m := []courseInfo{}

	count := 0
	c := colly.NewCollector()
	c.Limit(&colly.LimitRule{
		DomainGlob:  "*",
		Delay:       time.Duration(11 * time.Second),
		RandomDelay: time.Duration(1 * time.Second),
		Parallelism: 1,
	})
	c.DetectCharset = true

	c.OnHTML("div.lin-curso-c3 > a", func(e *colly.HTMLElement) {
		count++
		e.Request.Visit(e.Attr("href"))
	})

	c.OnHTML("div#caixa-orange", func(e *colly.HTMLElement) {
		course := courseInfo{}
		course.courseName = e.ChildText("div.cab1")
		course.institutionName = e.ChildText("div.cab2")
		course.examsInfo.etype = examsTypeMandatory
		examsStage := -1

		groupExams := false
		group := 0

		e.DOM.Find("div.inside2").Contents().Each(func(i int, s *goquery.Selection) {
			text := s.Text()
			if examsStage > 0 {
				if !s.Is("br") {
					if text != "" {
						if !strings.Contains(text, "ou") && !strings.Contains(text, "Observações") && !strings.Contains(text, "  e") {
							arr := strings.SplitN(text, " ", 2)

							if groupExams {
								if len(course.examsInfo.examGroups) <= group {
									code, _ := strconv.ParseUint(strings.TrimSpace(arr[0]), 10, 8) //TODO: Error handling
									course.examsInfo.examGroups = append(course.examsInfo.examGroups, []exam{{
										uint8(code),
										strings.TrimSpace(arr[1]),
									}})
								} else {
									code, _ := strconv.ParseUint(strings.TrimSpace(arr[0]), 10, 8) //TODO: Error handling
									course.examsInfo.examGroups[group] = append(course.examsInfo.examGroups[group], exam{
										uint8(code),
										strings.TrimSpace(arr[1]),
									})
								}
							} else {
								code, _ := strconv.ParseUint(strings.TrimSpace(arr[0]), 10, 8) //TODO: Error handling
								course.examsInfo.exams = append(course.examsInfo.exams, exam{
									uint8(code),
									strings.TrimSpace(arr[1]),
								})
							}
						} else {
							group++
						}
					}
					if s.Is("a") {
						examsStage = -1
					}
				}
			} else if examsStage == 0 {
				if strings.Contains(text, ":") {
					course.examsInfo.description = strings.TrimSpace(text)
					if strings.Contains(text, "conjuntos") {
						course.examsInfo.etype = examsTypeGroups
						groupExams = true
					}
				} else {
					course.examsInfo.etype = examsTypeUndefined
					course.examsInfo.description = "N/D"
					if text != "" {
						arr := strings.SplitN(text, " ", 2)
						code, _ := strconv.ParseUint(strings.TrimSpace(arr[0]), 10, 8) //TODO: Error handling
						course.examsInfo.exams = append(course.examsInfo.exams, exam{
							uint8(code),
							strings.TrimSpace(arr[1]),
						})
					}
				}
				examsStage = 1
			} else if s.Is("h2") && (text == "Provas de Ingresso") {
				examsStage = 0
			} else if strings.Contains(text, "Grau: ") {
				sub := strings.Split(text, "Grau: ")
				sube := strings.TrimSpace(sub[1])
				course.degree = sube
			} else if strings.Contains(text, "ECTS: ") {
				sub := strings.Split(text, "ECTS: ")
				sube := strings.TrimSpace(sub[1])
				ects, _ := strconv.ParseUint(sube, 10, 8) //TODO: Error handling
				course.ects = uint16(ects)
			} else if strings.Contains(text, "Área CNAEF: ") {
				sub := strings.Split(text, "Área CNAEF: ")
				sube := strings.TrimSpace(sub[1])
				course.cnaef = sube
			} else if strings.Contains(text, "Duração: ") {
				sub := strings.Split(text, "Duração: ")
				sube := strings.TrimSpace(sub[1])
				course.duration = sube
			} else if strings.Contains(text, "Código: ") {
				sub := strings.Split(text, "Código: ")
				sube := strings.TrimSpace(sub[1])
				splitSub := strings.Split(sube, "/")
				course.institutionCode = strings.TrimSpace(splitSub[0])
				course.courseCode = strings.TrimSpace(splitSub[1])
			} else if strings.Contains(text, "Tipo de Ensino: ") {
				sub := strings.Split(text, "Tipo de Ensino: ")
				sube := strings.TrimSpace(sub[1])
				course.ctype = sube
			} else if strings.Contains(text, "Concurso: ") {
				sub := strings.Split(text, "Concurso: ")
				sube := strings.TrimSpace(sub[1])
				course.contest = sube
			}
		})
		m = append(m, course)
	})

	c.OnRequest(func(r *colly.Request) {
		log.Println("Visiting", r.URL)
	})

	for r := 'a'; r <= 'z'; r++ {
		R := unicode.ToUpper(r)
		c.Visit("https://www.dges.gov.pt/guias/indcurso.asp?letra=" + string(R))
	}
	fmt.Println(count)

	{

		f, err := os.Create("data.txt")

		if err != nil {
			log.Fatal(err)
		}

		defer f.Close()

		for _, element := range m {
			f.WriteString("-------------------" + "\n")
			f.WriteString("CNAEF: " + element.cnaef + "\n")
			f.WriteString("Concurso: " + element.contest + "\n")
			f.WriteString("Curso: " + element.courseCode + "\n")
			f.WriteString("Nome do Curso: " + element.courseName + "\n")
			f.WriteString("Instituição: " + element.institutionCode + "\n")
			f.WriteString("Nome da Instituição: " + element.institutionName + "\n")
			f.WriteString("Typo de Ensino: " + element.ctype + "\n")
			f.WriteString("Grau: " + element.degree + "\n")
			f.WriteString("Duração: " + element.duration + "\n")
			f.WriteString("ECTS: " + fmt.Sprint(element.ects) + "\n")
			f.WriteString("Exams description: " + element.examsInfo.description + "\n")
			for _, exam := range element.examsInfo.exams {
				f.WriteString("Exam code: " + fmt.Sprint(exam.code) + "\n")
				f.WriteString("Exam name: " + exam.name + "\n")
			}
			for i, exams := range element.examsInfo.examGroups {
				f.WriteString(fmt.Sprint(i) + "\n")
				for _, exam := range exams {
					f.WriteString("Exam code: " + fmt.Sprint(exam.code) + "\n")
					f.WriteString("Exam name: " + exam.name + "\n")
				}
			}
			f.WriteString("-------------------" + "\n")
			f.Sync()
		}
	}

}

/*
Escolhe uma das seguintes provas
Escolhe um dos seguintes conjuntos
e
ou

...
**/

func parseExams(text string, s *goquery.Selection, examsStage int, groupExams bool, group int, course *courseInfo) {
	if examsStage > 0 {
		if !s.Is("br") {
			if text != "" {
				if !strings.Contains(text, "ou") && !strings.Contains(text, "Observações") && !strings.Contains(text, "  e") {
					arr := strings.SplitN(text, " ", 2)

					if groupExams {
						if len(course.examsInfo.examGroups) <= group {
							code, _ := strconv.ParseUint(strings.TrimSpace(arr[0]), 10, 8) //TODO: Error handling
							course.examsInfo.examGroups = append(course.examsInfo.examGroups, []exam{{
								uint8(code),
								strings.TrimSpace(arr[1]),
							}})
						} else {
							code, _ := strconv.ParseUint(strings.TrimSpace(arr[0]), 10, 8) //TODO: Error handling
							course.examsInfo.examGroups[group] = append(course.examsInfo.examGroups[group], exam{
								uint8(code),
								strings.TrimSpace(arr[1]),
							})
						}
					} else {
						code, _ := strconv.ParseUint(strings.TrimSpace(arr[0]), 10, 8) //TODO: Error handling
						course.examsInfo.exams = append(course.examsInfo.exams, exam{
							uint8(code),
							strings.TrimSpace(arr[1]),
						})
					}
				} else {
					group++
				}
			}
			if s.Is("a") {
				examsStage = -1
			}
		}
	} else if examsStage == 0 {
		if strings.Contains(text, ":") {
			course.examsInfo.description = strings.TrimSpace(text)
			if strings.Contains(text, "conjuntos") {
				course.examsInfo.etype = examsTypeGroups
				groupExams = true
			}
		} else {
			course.examsInfo.etype = examsTypeUndefined
			course.examsInfo.description = "N/D"
			if text != "" {
				arr := strings.SplitN(text, " ", 2)
				code, _ := strconv.ParseUint(strings.TrimSpace(arr[0]), 10, 8) //TODO: Error handling
				course.examsInfo.exams = append(course.examsInfo.exams, exam{
					uint8(code),
					strings.TrimSpace(arr[1]),
				})
			}
		}
		examsStage = 1
	}

}
