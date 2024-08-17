# Overview

The image shows the overall process works. The designed is based on a multi-agents technique [1] which they provide the idea to orchestate LLM agents to complete a task.

<img src="assets/Data ingestion (2).png">


According to the image, the platform include 2 processes:

1. [Data ingestion](01_data_ingestion.ipynb)
    * Scrape PDF from SFC
    * Extract data from PDF files
    * Upload to Opensearch
2. [Prompt development](02_prompt_development.ipynb)
    * Implementing workflow to response a question
# Set up environment

## Config `.env` file
1. Duplicate the file `.env.example` into `.env` with command:

```
cp .env.example .env
```
2. In the `.env`, configure environment variables.
```
export OPENSEARCH_USERNAME=OpenSearch admin username
export OPENSEARCH_PASSWORD=OpenSearch admin password
export OPENSEARCH_URL=OpenSearch URL, default to "http://localhost:9200"
```

## Opensearch
<img src="assets/opensearch_logo_default.svg">

To initiate an OpenSearch instant ensure `docker-compose.yml` is in the current directory, config file if needed and run

```{bash}
docker compose -d up --env-file=.env
```


## Ollama
<img src="assets/42f6b28d-9117-48cd-ac0d-44baaf5c178e.png" height=5 width=500>

1. Install Ollama by following the [instruction](https://ollama.com/download)
2. On the terminal run `ollama serve`
3. In this project, I decided to use `llama3:8b-instruct-q6_K` to be the main language model and `all-minilm` as the embedding model. To pull those models, run this command:
```{bash}
ollama pull llama3:8b-instruct-q6_K
ollama pull all-minilm
```

## Python
1. Python version in this project is `3.11`
2. install requiremens by run `pip install requirements.txt`

# Data ingestion

<img src="assets/Data ingestion (1).png">
The script to ingest into the OpenSearch instant is located in `01_data_ingestion.ipynb`. The script covers `data scraping`, `extract PDF data` and upload to `Opensearch`

1. Scrape URL link to PDF file:
I create a utility module storing a function to scrape the latest code of conduct pdf files on [SFC website](https://www.sfc.hk/en/Rules-and-standards/Codes-and-guidelines/Codes). These output URLs will be downloaded as bytes and input into the extractor.

```{python}
# file: utils/sfc_look_up.py

def extract_table() -> list[Content]:
    """Extract PDF files from the SFC website.

    Returns:
        list[Content]: Output PDF URLs
    """
    ...
    return table_of_content

```

2. Extract PDF using `pdfplumber`:

The script extract text from PDF files using simple text OCR techniques. Blank pages and too short pages are skipped.

3. Upload to OpenSearch
Before uploading in embeddeing text into a vector using `all-minilm`. I include some metadata to make easier query. To be uploaded document will be like:
```
{
    "file_url": file_url,
    "topic_title": topic_title,
    "page_number": page_number,
    "chunk_index": chunk_index,
    "text": chunk_text,
    "embedding": embedding,
}
```

# Prompt development
For prompt development, I use multi agents technique which initials many agents to handle different tasks and communicate to others.
<img src="assets/Data ingestion (4).png">


## Agents

### Topic selection
Match user's question to existant documents. If it doesn't match any forward to the `Apologize` agent

### Apologize:
If the process cannot find any resource to answer user's question, generate apologize text and send to the user

### Build query
Build OpenSearch query based on the selected topic by extract keyword from the question

### Check if results are summarize
Valid for each query result and consider if they are useful. If not, remove it. If all query results are not useful forward to the `Apologize` agent.

### Writing output agent
Use summarized contexts to be materials for writing text to response the user

## Workflow

```
def answer_the_question(question: str, stream: bool = False) -> Union[Generator[str, None, None], str]:
    topic_selected_index = topic_selector.pick_a_choice(question)
    if topic_selected_index < 1:
        # We don't have information to answer the question.
        return apologize(question, stream)
    topic_title = topic_selector.topic_choices[topic_selected_index - 1]
    query_text = text_query_builder.build(question)
    
    # Query
    search_text_results_raw = search_data_in_opensearch(
        query_text, search_method="text", topic_title=topic_title
    )
    search_vector_results_raw = search_data_in_opensearch(
        query_text, search_method="vector", topic_title=topic_title
    )
    search_text_results = extract_search_results(search_text_results_raw)
    search_vector_results = extract_search_results(search_vector_results_raw)

    # Summerize useful resources
    context_text_results = summarize_into_contexts(source_summarizer, search_text_results)
    context_vector_results = summarize_into_contexts(
        source_summarizer, search_vector_results
    )
    contexts = context_text_results + context_vector_results
    if len(contexts) == 0:
        # The retrieved resources are not useful. 
        return apologize(question, stream)
    # Generate the answer
    answer = user_interactor.answer(question, topic=topic_title, contexts=contexts, stream=stream)
    return answer
```

Funciton explaination:
1. Recieve user's question, and select the best document's topic to build the response. Stop the process it not found
2. Use the topic from previouse process and embedded question to build a query.
3. Validate query quality
4. Write a response

## Example:

**Question:**

I want to invest in real estates. What detail should I know?

**code:**
```
question: str = "I want to invest in real estates. What detail should I know?"
answer = answer_the_question(question, stream=True)
```


----

**output:**

<div class="output_subarea output_markdown rendered_html" dir="auto"><p>A great decision to invest in real estates! To help you make an informed investment, I'd like to highlight some key details from the Code on Real Estate Investment Trusts.</p>
<p>Firstly, when evaluating a Minority-owned Property, consider its ownership structure, divestment restrictions, and potential impact on property value (Page 30). This is crucial to ensure that your investment aligns with your goals and risk tolerance.</p>
<p>Secondly, be aware of the administrative requirements, laws governing repossession of properties, and tenant protection laws in the jurisdiction you're interested in investing in. Additionally, significant fluctuations in exchange rates between property valuation date and publication date may affect the valuation (Page 75).</p>
<p>Thirdly, familiarize yourself with the scheme's investment strategy. According to the Code, at least 75% of the gross asset value shall be invested in real estate that generates recurrent rental income at all times (Page 37). The scheme may also invest in uncompleted units or those undergoing development, redevelopment, or refurbishment, but not exceeding 25% of the gross asset value.</p>
<p>Fourthly, consider investing in another listed real estate investment trust if its structure, underlying investments, and regulatory regime are comparable to those in Hong Kong. It's recommended to consult with the Commission at an early stage for review (Page 43).</p>
<p>Lastly, when reviewing the offering document, look for a discussion of the business plan for property investment and management, including the type(s) of real estate invested in or intended to be made by the scheme (Page 78).</p>
<p>I hope these key takeaways help you make a more informed decision about your real estate investment. For more information, I recommend reviewing the Code on Real Estate Investment Trusts, available at <a href="https://www.sfc.hk/-/media/EN/files/COM/Reports-and-surveys/REIT-Code_Aug2022_en.pdf?rev=572cff969fc344fe8c375bcaab427f3b">https://www.sfc.hk/-/media/EN/files/COM/Reports-and-surveys/REIT-Code_Aug2022_en.pdf?rev=572cff969fc344fe8c375bcaab427f3b</a>.</p>
<p>Remember to always consult with a financial advisor or conduct your own research before making any investment decisions.<br><br><strong>Reference</strong></p>
<blockquote>
<p>From: <a href="https://www.sfc.hk/-/media/EN/files/COM/Reports-and-surveys/REIT-Code_Aug2022_en.pdf?rev=572cff969fc344fe8c375bcaab427f3b">https://www.sfc.hk/-/media/EN/files/COM/Reports-and-surveys/REIT-Code_Aug2022_en.pdf?rev=572cff969fc344fe8c375bcaab427f3b</a></p>
</blockquote>
<ul>
<li>"When valuing a Minority-owned Property, consider its ownership structure, divestment restrictions, and impact on property value. Also, note that the valuation requirements may not be strictly required when investing in other listed real estate investment trusts." - Page: 30</li>
<li>"The chapter highlights that the scheme primarily invests in real estate, with at least 75% of its gross asset value invested in income-generating properties. Additionally, it may acquire uncompleted units or those undergoing development, redevelopment, or refurbishment, but not exceeding 25% of the gross asset value." - Page: 35</li>
<li>"According to this text, you should know that at least 75% of the gross asset value of a scheme shall be invested in real estate that generates recurrent rental income at all times." - Page: 37</li>
<li>"Investing in another listed real estate investment trust may be regarded as a Qualified Minority-owned Property if its structure, underlying investments and regulatory regime are comparable to those in Hong Kong. It's recommended to consult the Commission at an early stage for review." - Page: 43</li>
<li>"You should know about administrative requirements, laws governing repossession of properties and tenant protection laws. Also, be aware that significant fluctuations in exchange rates between property valuation date and publication date may affect the valuation." - Page: 75</li>
<li>"The offering document should include a discussion of the business plan for property investment and management, covering the scope and type of investments made or intended to be made by the scheme, including the type(s) of real estate (e.g. residential/commercial/industrial)." - Page: 78</li>
</ul>
</div>

**Question:**

I want to design product about pooled Retirement funds, any suggestion?

**code:**
```
question: str = "I want to design product about pooled Retirement funds, any suggestion?"
answer = answer_the_question(question, stream=True)
```


----
**output:**

<div class="output_subarea output_markdown rendered_html" dir="auto"><p>Designing a product about pooled retirement funds! That's an exciting project.</p>
<p>Based on the prepared data source, I'd like to suggest some ideas:</p>
<ol>
<li><strong>Principal Brochure</strong>: Create a comprehensive principal brochure that outlines the key features and benefits of your pooled retirement fund product. This document should contain essential information about the investment portfolio(s), investment objectives, and risks associated with the fund.</li>
<li><strong>Investment Portfolio</strong>: Design an investment portfolio that aligns with the interests of participating ORSO schemes (Occupational Retirement Schemes). Consider diversifying the portfolio to minimize risk and maximize returns for retirement savings.</li>
<li><strong>Constituent Funds</strong>: Explore the possibility of creating constituent funds or pooled investment funds that cater to specific objectives, such as environmental, social, and governance (ESG) investing or impact investing. This could attract investors seeking socially responsible investments.</li>
<li><strong>Regulatory Compliance</strong>: Ensure your product complies with relevant regulations, including those set by the Securities and Futures Commission (SFC). Familiarize yourself with the Code on Pooled Retirement Funds and Section 104(1) of the Securities and Futures Ordinance (Cap. 571).</li>
<li><strong>Diversification and Risk Management</strong>: Consider incorporating strategies to manage risk and optimize returns, such as diversifying investments across asset classes, sectors, or geographic regions.</li>
<li><strong>Investor Education</strong>: Develop educational materials and resources to help investors understand the benefits and risks associated with pooled retirement funds. This could include workshops, webinars, or online tutorials.</li>
</ol>
<p>To learn more about designing a product for pooled retirement funds, I recommend exploring the Code on Pooled Retirement Funds (<a href="https://www.sfc.hk/-/media/EN/assets/components/codes/files-current/web/codes/code-on-pooled-retirement-funds/code-on-pooled-retirement-funds.pdf?rev=9badf81950734ee08c799832be6ff92b">https://www.sfc.hk/-/media/EN/assets/components/codes/files-current/web/codes/code-on-pooled-retirement-funds/code-on-pooled-retirement-funds.pdf?rev=9badf81950734ee08c799832be6ff92b</a>), specifically pages 12, 26, and the appendix.</p>
<p>For further information on pooled retirement funds, I suggest consulting with industry experts or regulatory bodies like the Securities and Futures Commission (SFC). You can also explore relevant academic research papers or articles on this topic.<br><br><strong>Reference</strong></p>
<blockquote>
<p>From: <a href="https://www.sfc.hk/-/media/EN/assets/components/codes/files-current/web/codes/code-on-pooled-retirement-funds/code-on-pooled-retirement-funds.pdf?rev=9badf81950734ee08c799832be6ff92b">https://www.sfc.hk/-/media/EN/assets/components/codes/files-current/web/codes/code-on-pooled-retirement-funds/code-on-pooled-retirement-funds.pdf?rev=9badf81950734ee08c799832be6ff92b</a></p>
</blockquote>
<ul>
<li>"The Code provides guidance on the authorization of collective investment schemes that are pooled retirement funds. According to the Commission's empowerment under section 104(1) of the Securities and Futures Ordinance (Cap. 571), they can authorize such schemes and impose conditions as deemed appropriate." - Page: 6</li>
<li>"The Securities and Futures Commission (SFC) authorizes collective investment schemes, including pooled retirement funds. The SFC may review its authorization at any time and impose conditions as it deems fit." - Page: 6</li>
<li>"This text defines terms related to Pooled Retirement Funds (PRFs) and Product Providers, which may be relevant for designing a product about PRFs. Specifically, it explains the meaning of 'pooled retirement fund', 'principal brochure', and 'product provider' in the context of the Occupational Retirement Schemes Ordinance." - Page: 12</li>
<li>"Section 3.8 defines an 'investment portfolio' as a constituent fund of a PRF into which assets of participating ORSO schemes are pooled for investment." - Page: 12</li>
<li>"Section 3.11 defines principal brochure or offering document as documents containing information on a PRF and investment portfolio(s) as stipulated in Appendix A, which may provide insights for designing a product about pooled Retirement funds." - Page: 12</li>
<li>"The constitutive document mentions that if a constituent fund or pooled investment fund indicates a particular objective, it should invest at least 70% of its total net asset value in securities and other investments to reflect the particular objective." - Page: 26</li>
</ul>
</div>

**Question:**

> Note: Testing a unrelated question

How to cook fried chicken?

**code:**
```
question: str = "How to cook fried chicken?"
answer = answer_the_question(question, stream=True)
```


----
**output:**


<div class="output_subarea output_markdown rendered_html" dir="auto"><p>I apologize, but I don't have any information on how to cook fried chicken. Unfortunately, we are a large language model and do not have personal experiences or recipes stored in our database.</p>
<p>However, I can suggest some helpful resources where you may find the answer to your question:</p>
<ol>
<li>Google Search: You can try searching for "how to cook fried chicken" on Google and it will show you various results with recipes and cooking methods.</li>
<li>Online Cooking Websites: Websites like Allrecipes, Epicurious, or Food.com have a wide range of fried chicken recipes that you can try.</li>
<li>Cookbooks: If you prefer learning from books, you can check out your local library or bookstore for cookbooks on Southern cuisine, comfort food, or fried chicken specifically.</li>
</ol>
<p>Remember to always follow proper cooking safety and hygiene practices when preparing food. I hope this helps, and please let me know if there's anything else I can assist you with!</p>
</div>

# Further improvements
## Scalablity

Interaction between agents in this project is conducted by passing-through function arguments. However, this could be improved for scalablity. For example, implementing a communication protocal using PUB/SUB or message queue technique allow the process can be scaled on-demand.

## Using different models for each agent

Some agents doesn't need high-performance model to complete their tasks. Their models can be replace with smaller model like `Phi` or `classication` models

## Use OCR for PDF extraction

Some information in a PDF file cannot be extracted due to its complicated structure like images or tables. I could suggest using Language and Vision Model like LLaVa is potentially fix this problem.

# Reference
[1] Large Language Model based Multi-Agents: A Survey of Progress and Challenges -  https://arxiv.org/abs/2402.01680<br>
[2] Creating Vector Database with OpenSearch - https://medium.com/marvelous-mlops/creating-vector-database-with-opensearch-7562b7451978
